//! Integration tests for payments: place_order creates payment intent, verify_razorpay_payment.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//! - For P2/P3, `RAZORPAY_KEY_SECRET` is set in the test (no real key needed).
//!
//! **Run**
//! - `cargo test --test integration_payments -- --ignored`

mod integration_common;
mod provider_test_gate;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::{
    FulfillmentStatus, PaymentStatus, Status as PaymentIntentStatus,
};
use core_db_entities::entity::{
    inventory, order_details, order_status, orders, payment_intents, product_categories,
    product_variants, products, shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use core_operations::procedures::stale_order_expiry::expire_stale_pending_orders;
use hmac::{Hmac, Mac};
use integration_common::test_db_url;
use proto::proto::core::{
    CreateCartItemRequest, CreateUserRequest, IngestWebhookRequest, PlaceOrderRequest,
    VerifyRazorpayPaymentRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbBackend, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Statement, TransactionTrait,
};
use sha2::Sha256;
use tonic::Request;

type HmacSha256 = Hmac<Sha256>;

async fn ensure_order_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Ok(Some(id)) = core_operations::order_state_machine::get_status_id(txn, name).await {
        return id;
    }
    let m = order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .expect("insert OrderStatus");
    m.status_id
}

/// Compute Razorpay-style signature: HMAC-SHA256(secret, "razorpay_order_id|razorpay_payment_id") hex-encoded.
fn compute_razorpay_signature(order_id: &str, payment_id: &str, secret: &str) -> String {
    let payload = format!("{}|{}", order_id, payment_id);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn restore_env_var(name: &str, previous: Option<String>) {
    if let Some(value) = previous {
        std::env::set_var(name, value);
    } else {
        std::env::remove_var(name);
    }
}

/// Ids of everything `place_order_setup` created, needed both by callers that mutate further
/// state (against a fresh transaction/connection obtained from `db`) and for the best-effort
/// cleanup they must now do themselves -- see the comment on `place_order_setup`.
struct PlaceOrderSetup {
    user_id: i64,
    order_id: i64,
    role_id: i64,
    category_id: i64,
    variant_id: i64,
}

/// Sets up a user + shipping address + product/variant/inventory + cart item, then calls
/// `place_order` to create a real order and payment intent for the caller to assert against.
///
/// Setup runs in its own transaction, committed just before calling `place_order`: place_order
/// now manages its own transactions internally (a short claim/prep transaction, then the
/// Shiprocket/Razorpay calls with no DB connection held, then a write transaction), so it can no
/// longer run as a nested savepoint inside a caller-supplied, still-open transaction -- the whole
/// point of that restructuring is that place_order's connection isn't held hostage across
/// external calls, which requires its transactions to be independently committable. Fixtures
/// must therefore already be committed before place_order's own prep phase reads them.
async fn place_order_setup(
    db: &DatabaseConnection,
    now_tag: i64,
    cart_total_paise: i64,
    payment_mode: Option<&str>,
) -> PlaceOrderSetup {
    let txn = db.begin().await.expect("begin setup transaction");

    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(&txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(&txn)
            .await
            .expect("insert pending OrderStatus");
    }

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_pay_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_pay_{}", now_tag),
            email: format!("itest_pay+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user");
    let user_id = user_res.into_inner().items[0].user_id;

    let ship = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some("Test User".to_string())),
        phone_number: ActiveValue::Set(Some("+919876543210".to_string())),
        is_default: ActiveValue::Set(0),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("City".to_string()),
        postal_code: ActiveValue::Set("100001".to_string()),
        road: ActiveValue::Set(None),
        apartment_no_or_name: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");
    let shipping_id = ship.shipping_address_id;

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        exchange_eligible: sea_orm::ActiveValue::Set(0),
        name: ActiveValue::Set(format!("itest_cat_pay_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Payment Test Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(cart_total_paise as i32),
        category_id: ActiveValue::Set(cat.category_id),
        fabric: ActiveValue::Set(None),
        weave: ActiveValue::Set(None),
        occasion: ActiveValue::Set(None),
        has_blouse_piece: ActiveValue::Set(None),
        care_instructions: ActiveValue::Set(None),
        product_status_id: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        updated_at: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(prod.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    let _ = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(10)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(&txn)
    .await
    .expect("insert Inventory");

    let cart_res = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item");
    let cart_id = cart_res.into_inner().items[0].cart_id;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions --
    // place_order no longer runs as a nested savepoint inside our transaction, so anything it
    // needs to read must already be committed.
    txn.commit()
        .await
        .expect("commit place_order_setup fixtures");

    let place_res = place_order(
        db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_id],
            payment_mode: payment_mode.map(|mode| mode.to_string()),
        }),
    )
    .await
    .expect("place_order");
    let order_id = place_res.into_inner().items[0].order_id;

    PlaceOrderSetup {
        user_id,
        order_id,
        role_id: role.role_id,
        category_id: cat.category_id,
        variant_id: variant.variant_id,
    }
}

/// Best-effort teardown of the fixture rows created by `place_order_setup` (and by the manual
/// inline setup in `integration_place_order_prepaid_rolls_back_fully_when_razorpay_fails`, which
/// mirrors it). This used to live inside the same uncommitted transaction as the place_order
/// call under test and simply never got committed on rollback; now that setup -- and
/// place_order's own transactions -- must commit independently (see `place_order_setup`),
/// cleanup needs to be explicit. Deletes in FK-safe order (children before parents). `order_id`
/// is `None` when place_order itself failed and never persisted an order. Errors are logged, not
/// fatal, so a cleanup failure never masks the actual test assertions.
async fn cleanup_payment_test_rows(
    db: &DatabaseConnection,
    order_id: Option<i64>,
    user_id: i64,
    role_id: i64,
    category_id: i64,
    variant_id: i64,
) {
    if let Some(order_id) = order_id {
        for (table, column) in [
            ("PaymentIntents", "order_id"),
            ("Shipments", "order_id"),
            ("OrderEvents", "order_id"),
            ("OrderDetails", "OrderID"),
            ("Orders", "OrderID"),
        ] {
            if let Err(e) = db
                .execute(Statement::from_sql_and_values(
                    DbBackend::MySql,
                    format!("DELETE FROM `{table}` WHERE `{column}` = ?"),
                    [order_id.into()],
                ))
                .await
            {
                eprintln!(
                    "warning: cleanup {table} for order_id={order_id} failed (non-fatal): {e}"
                );
            }
        }
    }

    let deletes: [(&str, &str, i64); 6] = [
        ("Cart", "UserID", user_id),
        ("Inventory", "VariantID", variant_id),
        ("ProductVariants", "VariantID", variant_id),
        ("Products", "CategoryID", category_id),
        ("ProductCategories", "CategoryID", category_id),
        ("ShippingAddresses", "UserID", user_id),
    ];
    for (table, column, id) in deletes {
        if let Err(e) = db
            .execute(Statement::from_sql_and_values(
                DbBackend::MySql,
                format!("DELETE FROM `{table}` WHERE `{column}` = ?"),
                [id.into()],
            ))
            .await
        {
            eprintln!("warning: cleanup {table} for id={id} failed (non-fatal): {e}");
        }
    }
    // Users last (ShippingAddresses/Cart above reference it), then UserRoles (Users references it).
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            "DELETE FROM `Users` WHERE `UserID` = ?",
            [user_id.into()],
        ))
        .await
    {
        eprintln!("warning: cleanup Users for user_id={user_id} failed (non-fatal): {e}");
    }
    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DbBackend::MySql,
            "DELETE FROM `UserRoles` WHERE `RoleID` = ?",
            [role_id.into()],
        ))
        .await
    {
        eprintln!("warning: cleanup UserRoles for role_id={role_id} failed (non-fatal): {e}");
    }
}

/// P1 – place_order creates a payment_intents row with correct order_id, amount, and pending status.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_creates_payment_intent() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_place_order_creates_payment_intent",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    // Keep subtotal above FREE_SHIPPING_THRESHOLD_MINOR so tests do not depend on live shipping quote.
    let cart_total = 150_000_i64;
    let fixture = place_order_setup(&db, now_tag, cart_total, None).await;
    let order_id = fixture.order_id;

    let intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .all(&db)
        .await
        .expect("query payment_intents");
    assert_eq!(
        intents.len(),
        1,
        "place_order should create exactly one payment intent"
    );
    let intent = &intents[0];
    assert_eq!(intent.order_id, Some(order_id));
    assert_eq!(intent.amount_paise, cart_total as i32);
    assert_eq!(intent.status, PaymentIntentStatus::Pending);
    assert!(intent.razorpay_payment_id.is_none());

    cleanup_payment_test_rows(
        &db,
        Some(order_id),
        fixture.user_id,
        fixture.role_id,
        fixture.category_id,
        fixture.variant_id,
    )
    .await;
}

/// P1b – If Razorpay order creation fails during a prepaid checkout, place_order
/// must leave nothing persisted: no Order row, no inventory decrement, and the
/// cart item stays put for the customer to retry. Forces a deterministic failure
/// by clearing RAZORPAY_KEY_ID (fails fast on missing config; no live network
/// call is made, so this test needs no provider-dependent gate).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_prepaid_rolls_back_fully_when_razorpay_fails() {
    // Trigger the one-time .env load first, then remove the Razorpay vars — if
    // removed before this, `load_env_once()`'s dotenv load (triggered by
    // `test_db_url()` below) would just repopulate them from `.env` afterward.
    core_operations::load_env_once();
    let original_key_id = std::env::var("RAZORPAY_KEY_ID").ok();
    let original_secret = std::env::var("RAZORPAY_KEY_SECRET").ok();
    std::env::remove_var("RAZORPAY_KEY_ID");
    std::env::remove_var("RAZORPAY_KEY_SECRET");

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let cart_total = 150_000_i64;

    // Setup runs in its own transaction, committed before calling place_order -- place_order
    // manages its own transactions internally now and can no longer see uncommitted fixtures
    // from a transaction it doesn't know about (see `place_order_setup` above for the full
    // rationale; this test builds its fixture inline instead of via that helper).
    let txn = db.begin().await.expect("begin transaction");

    let _ = ensure_order_status(&txn, "pending").await;
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_pay_fail_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_pay_fail_{}", now_tag),
            email: format!("itest_pay_fail+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user");
    let user_id = user_res.into_inner().items[0].user_id;

    let ship = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some("Test User".to_string())),
        phone_number: ActiveValue::Set(Some("+919876543210".to_string())),
        is_default: ActiveValue::Set(0),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("City".to_string()),
        postal_code: ActiveValue::Set("100001".to_string()),
        road: ActiveValue::Set(None),
        apartment_no_or_name: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");
    let shipping_id = ship.shipping_address_id;

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        exchange_eligible: sea_orm::ActiveValue::Set(0),
        name: ActiveValue::Set(format!("itest_cat_pay_fail_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");
    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Payment Failure Test Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(cart_total as i32),
        category_id: ActiveValue::Set(cat.category_id),
        fabric: ActiveValue::Set(None),
        weave: ActiveValue::Set(None),
        occasion: ActiveValue::Set(None),
        has_blouse_piece: ActiveValue::Set(None),
        care_instructions: ActiveValue::Set(None),
        product_status_id: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        updated_at: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert Products");
    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(prod.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");
    let _ = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(10)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(&txn)
    .await
    .expect("insert Inventory");

    let cart_res = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item");
    let cart_id = cart_res.into_inner().items[0].cart_id;

    // Commit setup so it's visible to place_order's own (separately-opened) transactions.
    txn.commit().await.expect("commit setup");

    let place_res = place_order(
        &db,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_id],
            payment_mode: None, // prepaid
        }),
    )
    .await;
    assert!(
        place_res.is_err(),
        "place_order should fail when Razorpay order creation fails"
    );

    // No manual rollback-and-reverify dance is needed here (unlike before this restructuring):
    // place_order's write transaction rolls back internally the moment the Razorpay call fails,
    // so by the time place_order returns Err, that transaction is already gone -- reads below via
    // a fresh transaction already reflect final, settled state. A fresh transaction (rather than
    // `&db` directly) is used only because `get_cart_items` requires a `&DatabaseTransaction`;
    // it's read-only, so it's simply rolled back afterward.
    let verify_txn = db.begin().await.expect("begin verify transaction");

    let orders_for_user = orders::Entity::find()
        .filter(orders::Column::UserId.eq(user_id))
        .count(&verify_txn)
        .await
        .expect("count orders");
    assert_eq!(
        orders_for_user, 0,
        "no order should be persisted when the Razorpay call fails"
    );

    let cart_after = core_operations::handlers::cart::get_cart_items(
        &verify_txn,
        Request::new(proto::proto::core::GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("get_cart_items");
    assert_eq!(
        cart_after.get_ref().items.len(),
        1,
        "cart item should remain after a failed prepaid checkout"
    );
    assert_eq!(cart_after.get_ref().items[0].cart_id, cart_id);

    let inv = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(Some(variant.variant_id)))
        .one(&verify_txn)
        .await
        .expect("query inventory")
        .expect("inventory row exists");
    assert_eq!(
        inv.quantity_available,
        Some(10),
        "inventory should not be decremented when the Razorpay call fails"
    );
    verify_txn.rollback().await.ok();

    restore_env_var("RAZORPAY_KEY_ID", original_key_id);
    restore_env_var("RAZORPAY_KEY_SECRET", original_secret);

    // No order was persisted (place_order failed), so cleanup only needs to remove the setup
    // fixtures that were committed above.
    cleanup_payment_test_rows(
        &db,
        None,
        user_id,
        role.role_id,
        cat.category_id,
        variant.variant_id,
    )
    .await;
}

/// P2 – Happy-path verify_razorpay_payment marks intent Processed, sets payment id, moves order to Paid.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_verify_razorpay_payment_success_updates_intent() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_verify_razorpay_payment_success_updates_intent",
    ) {
        return;
    }

    const TEST_SECRET: &str = "itest_razorpay_secret";
    let original_secret = std::env::var("RAZORPAY_KEY_SECRET").ok();

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_setup(&db, now_tag, 150_000, None).await;
    let order_id = fixture.order_id;

    // Fresh transaction for the verify_razorpay_payment call and its assertions --
    // place_order_setup's own setup transaction is already committed and gone by the time it
    // returns. Read-only from here except for the verify call itself, so it's rolled back at the
    // end; the order/payment-intent rows it touches are removed by explicit cleanup regardless.
    let txn = db.begin().await.expect("begin transaction");
    let confirmed_id = ensure_order_status(&txn, "confirmed").await;

    let intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query payment_intents");
    let intent = intents.into_iter().next().expect("one intent");
    let razorpay_order_id = intent.razorpay_order_id.clone();
    let razorpay_payment_id = "pay_test_verified_123".to_string();
    let signature =
        compute_razorpay_signature(&razorpay_order_id, &razorpay_payment_id, TEST_SECRET);

    std::env::set_var("RAZORPAY_KEY_SECRET", TEST_SECRET);
    let verify_res = core_operations::handlers::payment_intents::verify_razorpay_payment(
        &txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id,
            razorpay_order_id: razorpay_order_id.clone(),
            razorpay_payment_id: razorpay_payment_id.clone(),
            razorpay_signature: signature,
        }),
    )
    .await
    .expect("verify_razorpay_payment should not error");
    let inner = verify_res.into_inner();
    assert!(inner.verified, "signature valid => verified true");
    assert!(inner.payment_intent.is_some());

    let updated = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .filter(payment_intents::Column::RazorpayOrderId.eq(&razorpay_order_id))
        .one(&txn)
        .await
        .expect("query intent")
        .expect("intent exists");
    assert_eq!(updated.status, PaymentIntentStatus::Processed);
    assert_eq!(
        updated.razorpay_payment_id.as_deref(),
        Some(razorpay_payment_id.as_str())
    );

    let order_row = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(
        order_row.status_id, confirmed_id,
        "verify should promote order to Paid (confirmed)"
    );

    restore_env_var("RAZORPAY_KEY_SECRET", original_secret);
    txn.rollback().await.ok();

    cleanup_payment_test_rows(
        &db,
        Some(order_id),
        fixture.user_id,
        fixture.role_id,
        fixture.category_id,
        fixture.variant_id,
    )
    .await;
}

/// P3 – verify_razorpay_payment with invalid signature returns verification failure and does not update DB.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_verify_razorpay_payment_invalid_signature_no_update() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_verify_razorpay_payment_invalid_signature_no_update",
    ) {
        return;
    }

    const TEST_SECRET: &str = "itest_razorpay_secret_p3";
    let original_secret = std::env::var("RAZORPAY_KEY_SECRET").ok();

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_setup(&db, now_tag, 150_000, None).await;
    let order_id = fixture.order_id;

    // Fresh transaction, same reasoning as in the P2 test above.
    let txn = db.begin().await.expect("begin transaction");

    let intents = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query payment_intents");
    let intent = intents.into_iter().next().expect("one intent");
    let razorpay_order_id = intent.razorpay_order_id.clone();
    let razorpay_payment_id = "pay_test_invalid_456".to_string();

    std::env::set_var("RAZORPAY_KEY_SECRET", TEST_SECRET);
    let verify_res = core_operations::handlers::payment_intents::verify_razorpay_payment(
        &txn,
        Request::new(VerifyRazorpayPaymentRequest {
            order_id,
            razorpay_order_id: razorpay_order_id.clone(),
            razorpay_payment_id: razorpay_payment_id.clone(),
            razorpay_signature: "invalid_signature_hex".to_string(),
        }),
    )
    .await
    .expect("verify_razorpay_payment returns Ok even when signature invalid");
    let inner = verify_res.into_inner();
    assert!(!inner.verified);
    assert!(inner.payment_intent.is_none());

    let unchanged = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .filter(payment_intents::Column::RazorpayOrderId.eq(&razorpay_order_id))
        .one(&txn)
        .await
        .expect("query intent")
        .expect("intent exists");
    assert_eq!(unchanged.status, PaymentIntentStatus::Pending);
    assert!(unchanged.razorpay_payment_id.is_none());

    restore_env_var("RAZORPAY_KEY_SECRET", original_secret);
    txn.rollback().await.ok();

    cleanup_payment_test_rows(
        &db,
        Some(order_id),
        fixture.user_id,
        fixture.role_id,
        fixture.category_id,
        fixture.variant_id,
    )
    .await;
}

/// P4A - stale unpaid prepaid orders expire even after cancel window closes.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and shipping quote configuration"]
async fn integration_stale_unpaid_order_expiry_restores_inventory() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_stale_unpaid_order_expiry_restores_inventory",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_setup(&db, now_tag, 150_000, None).await;
    let order_id = fixture.order_id;

    // Fresh transaction for these mutations -- place_order_setup's own setup transaction is
    // already committed and gone by the time it returns. This one is committed below (not
    // rolled back) because expire_stale_pending_orders runs against `db` on its own connection
    // and needs to see these changes, same as before this restructuring.
    let txn = db.begin().await.expect("begin transaction");

    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .one(&txn)
        .await
        .expect("query payment intent")
        .expect("payment intent exists");
    let mut intent_active: payment_intents::ActiveModel = intent.clone().into();
    intent_active.expires_at = ActiveValue::Set(Utc::now() - chrono::Duration::hours(1));
    intent_active
        .update(&txn)
        .await
        .expect("expire payment intent");

    let mut order_active: orders::ActiveModel = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists")
        .into();
    order_active.cancel_window_ends_at =
        ActiveValue::Set(Some(Utc::now() - chrono::Duration::hours(2)));
    order_active
        .update(&txn)
        .await
        .expect("close customer cancel window");
    txn.commit().await.expect("commit setup");

    let expired = expire_stale_pending_orders(&db, 10)
        .await
        .expect("expire stale orders should succeed");
    assert_eq!(expired, 1);
    let repeat = expire_stale_pending_orders(&db, 10)
        .await
        .expect("repeat stale expiry should succeed");
    assert_eq!(
        repeat, 0,
        "second system-expiry run should be idempotent no-op"
    );

    let verify_txn = db.begin().await.expect("begin verify transaction");
    let updated_intent = payment_intents::Entity::find_by_id(intent.intent_id)
        .one(&verify_txn)
        .await
        .expect("query updated intent")
        .expect("updated intent exists");
    assert_eq!(updated_intent.status, PaymentIntentStatus::Failed);

    let cancelled_id = ensure_order_status(&verify_txn, "cancelled").await;
    let order = orders::Entity::find_by_id(order_id)
        .one(&verify_txn)
        .await
        .expect("query expired order")
        .expect("order exists");
    assert_eq!(order.status_id, cancelled_id);
    assert_eq!(order.payment_status, Some(PaymentStatus::Failed));
    assert_eq!(order.fulfillment_status, FulfillmentStatus::NotCreated);

    let cancelled_lines = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order_id))
        .filter(order_details::Column::ItemStatus.eq("cancelled"))
        .count(&verify_txn)
        .await
        .expect("count cancelled lines");
    assert_eq!(cancelled_lines, 1, "line items should be system-cancelled");

    let inventory_row = inventory::Entity::find()
        .filter(inventory::Column::VariantId.is_not_null())
        .order_by_desc(inventory::Column::InventoryId)
        .one(&verify_txn)
        .await
        .expect("query inventory")
        .expect("inventory exists");
    assert_eq!(inventory_row.quantity_available, Some(10));

    verify_txn.rollback().await.ok();

    cleanup_payment_test_rows(
        &db,
        Some(order_id),
        fixture.user_id,
        fixture.role_id,
        fixture.category_id,
        fixture.variant_id,
    )
    .await;
}

/// P4B - stale unpaid prepaid orders expire before cancel window too (system path ignores customer window).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and shipping quote configuration"]
async fn integration_stale_unpaid_order_expiry_before_cancel_window_still_succeeds() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_stale_unpaid_order_expiry_before_cancel_window_still_succeeds",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_setup(&db, now_tag, 150_000, None).await;
    let order_id = fixture.order_id;

    // Fresh, committed transaction -- see the comment in P4A above.
    let txn = db.begin().await.expect("begin transaction");
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .one(&txn)
        .await
        .expect("query payment intent")
        .expect("payment intent exists");
    let mut intent_active: payment_intents::ActiveModel = intent.clone().into();
    intent_active.expires_at = ActiveValue::Set(Utc::now() - chrono::Duration::hours(1));
    intent_active
        .update(&txn)
        .await
        .expect("expire payment intent");
    txn.commit().await.expect("commit setup");

    let expired = expire_stale_pending_orders(&db, 10)
        .await
        .expect("expire stale orders should succeed");
    assert_eq!(expired, 1);

    let verify_txn = db.begin().await.expect("verify txn");
    let cancelled_id = ensure_order_status(&verify_txn, "cancelled").await;
    let order = orders::Entity::find_by_id(order_id)
        .one(&verify_txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, cancelled_id);
    assert_eq!(order.payment_status, Some(PaymentStatus::Failed));
    verify_txn.rollback().await.ok();

    cleanup_payment_test_rows(
        &db,
        Some(order_id),
        fixture.user_id,
        fixture.role_id,
        fixture.category_id,
        fixture.variant_id,
    )
    .await;
}

/// P4C - COD orders are excluded from stale unpaid prepaid expiry path.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and shipping quote configuration"]
async fn integration_stale_unpaid_order_expiry_skips_cod_orders() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_stale_unpaid_order_expiry_skips_cod_orders",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_setup(&db, now_tag, 150_000, Some("cod")).await;
    let user_id = fixture.user_id;
    let order_id = fixture.order_id;

    // Fresh, committed transaction -- see the comment in P4A above.
    let txn = db.begin().await.expect("begin transaction");
    let synthetic_intent = payment_intents::ActiveModel {
        intent_id: ActiveValue::NotSet,
        razorpay_order_id: ActiveValue::Set(format!("order_cod_stale_{now_tag}")),
        order_id: ActiveValue::Set(Some(order_id)),
        active_order_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        amount_paise: ActiveValue::Set(150_000),
        currency: ActiveValue::Set(Some("INR".to_string())),
        status: ActiveValue::Set(PaymentIntentStatus::Pending),
        razorpay_payment_id: ActiveValue::Set(None),
        metadata: ActiveValue::Set(None),
        gateway_fee_paise: ActiveValue::Set(None),
        gateway_tax_paise: ActiveValue::Set(None),
        created_at: ActiveValue::Set(Some(Utc::now())),
        expires_at: ActiveValue::Set(Utc::now() - chrono::Duration::hours(1)),
    };
    let _ = synthetic_intent
        .insert(&txn)
        .await
        .expect("insert synthetic cod intent");
    txn.commit().await.expect("commit setup");

    let expired = expire_stale_pending_orders(&db, 10)
        .await
        .expect("expire stale orders should succeed");
    assert_eq!(expired, 0, "COD orders must not be system-expired");

    let verify_txn = db.begin().await.expect("verify txn");
    let confirmed_id = ensure_order_status(&verify_txn, "confirmed").await;
    let order = orders::Entity::find_by_id(order_id)
        .one(&verify_txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, confirmed_id);
    assert_eq!(
        order.payment_method.as_deref(),
        Some("cod"),
        "COD marker should remain unchanged"
    );
    verify_txn.rollback().await.ok();

    cleanup_payment_test_rows(
        &db,
        Some(order_id),
        fixture.user_id,
        fixture.role_id,
        fixture.category_id,
        fixture.variant_id,
    )
    .await;
}

/// P4D - captured/paid orders are not expired by stale unpaid worker even if a stale pending intent exists.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and shipping quote configuration"]
async fn integration_stale_unpaid_order_expiry_skips_captured_orders() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_stale_unpaid_order_expiry_skips_captured_orders",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_setup(&db, now_tag, 150_000, None).await;
    let order_id = fixture.order_id;

    // Fresh, committed transaction -- see the comment in P4A above.
    let txn = db.begin().await.expect("begin transaction");
    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let mut order_active: orders::ActiveModel = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists")
        .into();
    order_active.status_id = ActiveValue::Set(confirmed_id);
    order_active.payment_status = ActiveValue::Set(Some(PaymentStatus::Captured));
    order_active.update(&txn).await.expect("set order captured");

    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .one(&txn)
        .await
        .expect("query intent")
        .expect("intent exists");
    let mut intent_active: payment_intents::ActiveModel = intent.clone().into();
    intent_active.expires_at = ActiveValue::Set(Utc::now() - chrono::Duration::hours(1));
    intent_active.update(&txn).await.expect("expire intent");
    txn.commit().await.expect("commit setup");

    let expired = expire_stale_pending_orders(&db, 10)
        .await
        .expect("expire stale orders should succeed");
    assert_eq!(expired, 0, "captured orders must not be system-expired");

    let verify_txn = db.begin().await.expect("verify txn");
    let order = orders::Entity::find_by_id(order_id)
        .one(&verify_txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, confirmed_id);
    assert_eq!(order.payment_status, Some(PaymentStatus::Captured));
    verify_txn.rollback().await.ok();

    cleanup_payment_test_rows(
        &db,
        Some(order_id),
        fixture.user_id,
        fixture.role_id,
        fixture.category_id,
        fixture.variant_id,
    )
    .await;
}

/// P4E - late captured webhook after system expiry is flagged for manual review, not auto-paid.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL, migrated schema, and shipping quote configuration"]
async fn integration_late_captured_webhook_after_system_expiry_marks_needs_review() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_late_captured_webhook_after_system_expiry_marks_needs_review",
    ) {
        return;
    }

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let fixture = place_order_setup(&db, now_tag, 150_000, None).await;
    let order_id = fixture.order_id;

    // Fresh, committed transaction -- see the comment in P4A above.
    let txn = db.begin().await.expect("begin transaction");
    let intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order_id))
        .one(&txn)
        .await
        .expect("query payment intent")
        .expect("payment intent exists");
    let mut intent_active: payment_intents::ActiveModel = intent.clone().into();
    intent_active.expires_at = ActiveValue::Set(Utc::now() - chrono::Duration::hours(1));
    intent_active
        .update(&txn)
        .await
        .expect("expire payment intent");
    txn.commit().await.expect("commit setup");

    let expired = expire_stale_pending_orders(&db, 10)
        .await
        .expect("expire stale orders should succeed");
    assert_eq!(expired, 1);

    let webhook_txn = db.begin().await.expect("begin webhook txn");
    let payload = serde_json::json!({
        "event": "payment.captured",
        "payload": {
            "payment": {
                "entity": {
                    "id": format!("pay_late_{}", now_tag),
                    "order_id": intent.razorpay_order_id,
                    "amount": 150_000,
                    "currency": "INR"
                }
            }
        }
    });
    let webhook_id = format!("razorpay:late_capture:{now_tag}");
    let ingest = core_operations::handlers::webhooks::ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id: webhook_id.clone(),
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: None,
            raw_signature: None,
        }),
    )
    .await;
    assert!(
        ingest.is_ok(),
        "late capture webhook should be accepted and flagged for review"
    );
    let first_event = ingest
        .expect("already asserted ingest ok")
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("first webhook response item");

    let replay = core_operations::handlers::webhooks::ingest_webhook(
        &webhook_txn,
        Request::new(IngestWebhookRequest {
            provider: "razorpay".to_string(),
            event_type: "payment.captured".to_string(),
            webhook_id,
            payload_json: payload.to_string(),
            signature_verified: true,
            provider_event_id: None,
            raw_signature: None,
        }),
    )
    .await
    .expect("duplicate replay should be idempotent")
    .into_inner()
    .items
    .into_iter()
    .next()
    .expect("replay webhook response item");
    assert_eq!(replay.event_id, first_event.event_id);
    assert_eq!(replay.status, "processed");

    let refreshed_intent = payment_intents::Entity::find_by_id(intent.intent_id)
        .one(&webhook_txn)
        .await
        .expect("query refreshed intent")
        .expect("intent exists");
    assert_eq!(
        refreshed_intent.status,
        PaymentIntentStatus::NeedsReview,
        "late capture after system expiry must require manual review"
    );

    let cancelled_id = ensure_order_status(&webhook_txn, "cancelled").await;
    let refreshed_order = orders::Entity::find_by_id(order_id)
        .one(&webhook_txn)
        .await
        .expect("query refreshed order")
        .expect("order exists");
    assert_eq!(
        refreshed_order.status_id, cancelled_id,
        "late capture must not auto-promote cancelled order to confirmed"
    );

    webhook_txn.rollback().await.ok();

    cleanup_payment_test_rows(
        &db,
        Some(order_id),
        fixture.user_id,
        fixture.role_id,
        fixture.category_id,
        fixture.variant_id,
    )
    .await;
}
