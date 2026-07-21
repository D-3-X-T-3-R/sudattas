//! Integration test for place_order atomicity when one OrderDetails insert fails.
//!
//! Run:
//! `cargo test --test integration_place_order_atomicity integration_place_order_fails_on_partial_order_details_insert -- --ignored --nocapture`

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::{
    cart, inventory, order_status, product_categories, product_variants, products,
    shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use integration_common::test_db_url;
use proto::proto::core::{CreateCartItemRequest, CreateUserRequest, PlaceOrderRequest};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DatabaseTransaction, DbBackend, EntityTrait, PaginatorTrait, QueryFilter, Statement,
    TransactionTrait,
};
use tonic::{Code, Request};
use warp::Filter;

const FORCED_FAIL_TITLE: &str = "__itest_force_order_details_insert_failure__";

/// Prepaid `place_order` calls out to Razorpay to create the order before OrderDetails is
/// inserted. CI (and any environment without real Razorpay creds) has no RAZORPAY_KEY_ID, so
/// without this, prepaid runs fail on "Unable to create Razorpay order" instead of exercising
/// the OrderDetails-insert-failure path this test is actually about. Mirrors the local mock
/// server pattern from `razorpay_connectivity.rs` so this test never depends on real credentials.
async fn ensure_razorpay_mock_for_prepaid(payment_mode: &str) {
    if payment_mode != "prepaid" {
        return;
    }
    let orders = warp::path!("v1" / "orders")
        .and(warp::post())
        .map(|| warp::reply::json(&serde_json::json!({ "id": "order_itest_atomicity" })));
    let (addr, server) = warp::serve(orders).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::task::spawn(server);
    std::env::set_var("RAZORPAY_KEY_ID", "rzp_test_itest_atomicity");
    std::env::set_var("RAZORPAY_KEY_SECRET", "itest_atomicity_secret");
    std::env::set_var("RAZORPAY_API_BASE", format!("http://{}/v1", addr));
}

async fn ensure_order_status(txn: &DatabaseTransaction, name: &str) -> Result<i64, String> {
    if let Some(existing) = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq(name))
        .one(txn)
        .await
        .map_err(|e| format!("query OrderStatus `{name}`: {e}"))?
    {
        return Ok(existing.status_id);
    }

    let created = order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(name.to_string()),
    }
    .insert(txn)
    .await
    .map_err(|e| format!("insert OrderStatus `{name}`: {e}"))?;

    Ok(created.status_id)
}

async fn create_order_details_failure_trigger(
    db: &DatabaseConnection,
    trigger_name: &str,
) -> Result<(), String> {
    let drop_sql = format!("DROP TRIGGER IF EXISTS `{trigger_name}`");
    db.execute_unprepared(&drop_sql)
        .await
        .map_err(|e| format!("drop pre-existing trigger `{trigger_name}`: {e}"))?;

    let create_sql = format!(
        r#"CREATE TRIGGER `{trigger_name}`
BEFORE INSERT ON `OrderDetails`
FOR EACH ROW
BEGIN
    IF NEW.title = '{FORCED_FAIL_TITLE}' THEN
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'itest forced OrderDetails insert failure';
    END IF;
END"#
    );

    db.execute_unprepared(&create_sql)
        .await
        .map_err(|e| format!("create trigger `{trigger_name}`: {e}"))?;

    Ok(())
}

async fn drop_trigger(db: &DatabaseConnection, trigger_name: &str) -> Result<(), String> {
    let drop_sql = format!("DROP TRIGGER IF EXISTS `{trigger_name}`");
    db.execute_unprepared(&drop_sql)
        .await
        .map_err(|e| format!("drop trigger `{trigger_name}`: {e}"))?;
    Ok(())
}

async fn count_orders_by_order_id(txn: &DatabaseTransaction, order_id: i64) -> Result<i64, String> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM Orders WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| format!("count Orders by order_id={order_id}: {e}"))?
        .ok_or_else(|| "missing count row for Orders".to_string())?;
    row.try_get::<i64>("", "count")
        .map_err(|e| format!("read Orders count: {e}"))
}

async fn count_order_details_by_order_id(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<i64, String> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM OrderDetails WHERE OrderID = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| format!("count OrderDetails by order_id={order_id}: {e}"))?
        .ok_or_else(|| "missing count row for OrderDetails".to_string())?;
    row.try_get::<i64>("", "count")
        .map_err(|e| format!("read OrderDetails count: {e}"))
}

async fn count_orders_for_user(txn: &DatabaseTransaction, user_id: i64) -> Result<i64, String> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM Orders WHERE UserID = ?"#,
            [user_id.into()],
        ))
        .await
        .map_err(|e| format!("count Orders for user_id={user_id}: {e}"))?
        .ok_or_else(|| "missing count row for Orders by user".to_string())?;
    row.try_get::<i64>("", "count")
        .map_err(|e| format!("read Orders-by-user count: {e}"))
}

async fn latest_order_id_for_user(
    txn: &DatabaseTransaction,
    user_id: i64,
) -> Result<Option<i64>, String> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT OrderID AS order_id FROM Orders WHERE UserID = ? ORDER BY OrderID DESC LIMIT 1"#,
            [user_id.into()],
        ))
        .await
        .map_err(|e| format!("query latest order_id for user_id={user_id}: {e}"))?;

    Ok(row.and_then(|r| r.try_get::<i64>("", "order_id").ok()))
}

async fn count_payment_intents_by_order_id(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<i64, String> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM PaymentIntents WHERE order_id = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| format!("count payment_intents by order_id={order_id}: {e}"))?
        .ok_or_else(|| "missing count row for payment_intents".to_string())?;
    row.try_get::<i64>("", "count")
        .map_err(|e| format!("read payment_intents count: {e}"))
}

async fn count_shipments_by_order_id(
    txn: &DatabaseTransaction,
    order_id: i64,
) -> Result<i64, String> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            r#"SELECT COUNT(*) AS count FROM Shipments WHERE order_id = ?"#,
            [order_id.into()],
        ))
        .await
        .map_err(|e| format!("count Shipments by order_id={order_id}: {e}"))?
        .ok_or_else(|| "missing count row for Shipments".to_string())?;
    row.try_get::<i64>("", "count")
        .map_err(|e| format!("read Shipments count: {e}"))
}

async fn count_selected_cart_rows(
    txn: &DatabaseTransaction,
    cart_ids: &[i64],
) -> Result<i64, String> {
    let count = cart::Entity::find()
        .filter(cart::Column::CartId.is_in(cart_ids.iter().copied()))
        .count(txn)
        .await
        .map_err(|e| format!("count selected cart rows: {e}"))?;
    Ok(count as i64)
}

async fn inventory_available(txn: &DatabaseTransaction, variant_id: i64) -> Result<i64, String> {
    let row = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(Some(variant_id)))
        .one(txn)
        .await
        .map_err(|e| format!("query inventory for variant_id={variant_id}: {e}"))?
        .ok_or_else(|| format!("missing inventory row for variant_id={variant_id}"))?;
    row.quantity_available
        .ok_or_else(|| format!("inventory.quantity_available is NULL for variant_id={variant_id}"))
}

async fn inventory_snapshot(
    txn: &DatabaseTransaction,
    variant_ids: &[i64],
) -> Result<Vec<(i64, i64)>, String> {
    let mut out = Vec::with_capacity(variant_ids.len());
    for variant_id in variant_ids {
        out.push((*variant_id, inventory_available(txn, *variant_id).await?));
    }
    Ok(out)
}

async fn run_atomicity_scenario(db: &DatabaseConnection, payment_mode: &str) -> Result<(), String> {
    ensure_razorpay_mock_for_prepaid(payment_mode).await;

    let txn = db
        .begin()
        .await
        .map_err(|e| format!("begin transaction: {e}"))?;
    let now_tag = Utc::now().timestamp_millis();

    ensure_order_status(&txn, "pending").await?;
    ensure_order_status(&txn, "active_sale").await?;
    ensure_order_status(&txn, "confirmed").await?;

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_atomic_role_{now_tag}")),
    }
    .insert(&txn)
    .await
    .map_err(|e| format!("insert role: {e}"))?;

    let user = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_atomic_user_{now_tag}"),
            email: format!("itest_atomic+{now_tag}@example.com"),
            full_name: Some("Atomicity Test User".to_string()),
            address: Some("123 Atomicity Lane".to_string()),
            phone: Some(format!("+91{:010}", (now_tag.abs() % 10_000_000_000_i64))),
            auth_provider: "email".to_string(),
            password_plain: Some("StrongPass123!".to_string()),
            google_sub: None,
            role_id: Some(role.role_id),
        }),
    )
    .await
    .map_err(|e| format!("create user: {e}"))?
    .into_inner()
    .items
    .into_iter()
    .next()
    .ok_or_else(|| "create_user returned empty items".to_string())?;
    let user_id = user.user_id;

    let shipping = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        recipient_name: ActiveValue::Set(Some("Atomicity Recipient".to_string())),
        phone_number: ActiveValue::Set(Some("+919876543210".to_string())),
        is_default: ActiveValue::Set(1),
        country: ActiveValue::Set("IN".to_string()),
        state_region: ActiveValue::Set("KA".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(Some("Test Road".to_string())),
        apartment_no_or_name: ActiveValue::Set(Some("A-101".to_string())),
    }
    .insert(&txn)
    .await
    .map_err(|e| format!("insert shipping address: {e}"))?;

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_atomic_category_{now_tag}")),
    }
    .insert(&txn)
    .await
    .map_err(|e| format!("insert product category: {e}"))?;

    let product_titles = [
        "Atomicity Valid Product 1".to_string(),
        "Atomicity Valid Product 2".to_string(),
        FORCED_FAIL_TITLE.to_string(),
    ];
    let quantities = [1_i64, 1_i64, 7_i64];
    let mut variant_ids = Vec::with_capacity(3);
    let mut selected_cart_ids = Vec::with_capacity(3);

    for (idx, (title, qty)) in product_titles.iter().zip(quantities.iter()).enumerate() {
        let product = products::ActiveModel {
            product_id: ActiveValue::NotSet,
            sku: ActiveValue::Set(None),
            name: ActiveValue::Set(title.clone()),
            slug: ActiveValue::Set(None),
            description: ActiveValue::Set(None),
            price_paise: ActiveValue::Set(90_000 + (idx as i32 * 10_000)),
            category_id: ActiveValue::Set(category.category_id),
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
        .map_err(|e| format!("insert product[{idx}]: {e}"))?;

        let variant = product_variants::ActiveModel {
            variant_id: ActiveValue::NotSet,
            product_id: ActiveValue::Set(product.product_id),
            size_id: ActiveValue::Set(None),
            color_id: ActiveValue::Set(None),
            additional_price: ActiveValue::Set(Some(0)),
        }
        .insert(&txn)
        .await
        .map_err(|e| format!("insert variant[{idx}]: {e}"))?;

        inventory::ActiveModel {
            inventory_id: ActiveValue::NotSet,
            variant_id: ActiveValue::Set(Some(variant.variant_id)),
            quantity_available: ActiveValue::Set(Some(50)),
            quantity_reserved: ActiveValue::Set(Some(0)),
            reorder_level: ActiveValue::Set(None),
            updated_at: ActiveValue::Set(Some(Utc::now())),
        }
        .insert(&txn)
        .await
        .map_err(|e| format!("insert inventory[{idx}]: {e}"))?;

        let cart_item = core_operations::handlers::cart::create_cart_item(
            &txn,
            Request::new(CreateCartItemRequest {
                user_id: Some(user_id),
                session_id: None,
                variant_id: variant.variant_id,
                quantity: *qty,
            }),
        )
        .await
        .map_err(|e| format!("create cart item[{idx}]: {e}"))?
        .into_inner()
        .items
        .into_iter()
        .next()
        .ok_or_else(|| format!("create_cart_item[{idx}] returned empty items"))?;

        variant_ids.push(variant.variant_id);
        selected_cart_ids.push(cart_item.cart_id);
    }

    let expected_line_item_count = selected_cart_ids.len() as i64;
    let pre_inventory = inventory_snapshot(&txn, &variant_ids).await?;
    let pre_selected_cart_rows = count_selected_cart_rows(&txn, &selected_cart_ids).await?;
    if pre_selected_cart_rows != expected_line_item_count {
        return Err(format!(
            "setup failed: expected {expected_line_item_count} selected cart rows, got {pre_selected_cart_rows}"
        ));
    }

    let place_res = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: selected_cart_ids.clone(),
            payment_mode: Some(payment_mode.to_string()),
        }),
    )
    .await;

    let order_id_from_response = place_res
        .as_ref()
        .ok()
        .and_then(|resp| resp.get_ref().items.first().map(|o| o.order_id));
    let order_id_from_db = latest_order_id_for_user(&txn, user_id).await?;
    let observed_order_id = order_id_from_response.or(order_id_from_db);
    let count_query_order_id = observed_order_id.unwrap_or(-1);

    // Required explicit DB verification queries by order_id:
    let orders_count_for_order_id = count_orders_by_order_id(&txn, count_query_order_id).await?;
    let order_details_count_for_order_id =
        count_order_details_by_order_id(&txn, count_query_order_id).await?;

    let orders_count_for_user = count_orders_for_user(&txn, user_id).await?;
    let payment_intents_count =
        count_payment_intents_by_order_id(&txn, count_query_order_id).await?;
    let shipments_count = count_shipments_by_order_id(&txn, count_query_order_id).await?;
    let post_inventory = inventory_snapshot(&txn, &variant_ids).await?;
    let post_selected_cart_rows = count_selected_cart_rows(&txn, &selected_cart_ids).await?;

    // Transaction-boundary validation: explicit rollback must remove Orders + OrderDetails rows.
    txn.rollback()
        .await
        .map_err(|e| format!("rollback checkout txn: {e}"))?;

    if let Some(order_id) = observed_order_id {
        let verify_txn = db
            .begin()
            .await
            .map_err(|e| format!("begin verify rollback txn: {e}"))?;
        let orders_after_rollback = count_orders_by_order_id(&verify_txn, order_id).await?;
        let details_after_rollback = count_order_details_by_order_id(&verify_txn, order_id).await?;
        verify_txn
            .rollback()
            .await
            .map_err(|e| format!("rollback verify txn: {e}"))?;
        if orders_after_rollback != 0 || details_after_rollback != 0 {
            return Err(format!(
                "rollback boundary failure: order_id={order_id}, Orders after rollback={orders_after_rollback}, OrderDetails after rollback={details_after_rollback}"
            ));
        }
    }

    // Case B: bug exists (place_order returns Ok and persists partial details)
    if let Ok(ok_resp) = &place_res {
        let ok_order_id = ok_resp
            .get_ref()
            .items
            .first()
            .map(|o| o.order_id)
            .unwrap_or(-1);
        return Err(format!(
            "BUG: place_order returned Ok (payment_mode={payment_mode}, order_id={ok_order_id}) after forced OrderDetails failure. Orders count by order_id={orders_count_for_order_id}, OrderDetails count by order_id={order_details_count_for_order_id}, expected line items={expected_line_item_count}, inventory before={pre_inventory:?}, inventory after={post_inventory:?}, selected cart rows before={pre_selected_cart_rows}, selected cart rows after={post_selected_cart_rows}, payment_intents by order_id={payment_intents_count}, shipments by order_id={shipments_count}"
        ));
    }

    let err = place_res.expect_err("checked Err branch");
    if err.code() != Code::Internal && err.code() != Code::Unknown {
        return Err(format!(
            "expected place_order error code Internal/Unknown, got {:?} ({})",
            err.code(),
            err.message()
        ));
    }

    // Case A assertions (correct atomic behavior):
    if orders_count_for_user != 0 {
        return Err(format!(
            "atomicity violation: expected 0 Orders rows for user_id={user_id}, got {orders_count_for_user}"
        ));
    }
    if orders_count_for_order_id != 0 {
        return Err(format!(
            "atomicity violation: expected 0 Orders rows for order_id={count_query_order_id}, got {orders_count_for_order_id}"
        ));
    }
    if order_details_count_for_order_id != 0 {
        return Err(format!(
            "atomicity violation: expected 0 OrderDetails rows for order_id={count_query_order_id}, got {order_details_count_for_order_id}"
        ));
    }
    if payment_intents_count != 0 {
        return Err(format!(
            "atomicity violation: expected 0 payment_intents rows for order_id={count_query_order_id}, got {payment_intents_count}"
        ));
    }
    if shipments_count != 0 {
        return Err(format!(
            "atomicity violation: expected 0 Shipments rows for order_id={count_query_order_id}, got {shipments_count}"
        ));
    }
    if pre_inventory != post_inventory {
        return Err(format!(
            "atomicity violation: inventory changed on failed place_order. before={pre_inventory:?}, after={post_inventory:?}"
        ));
    }
    if post_selected_cart_rows != pre_selected_cart_rows {
        return Err(format!(
            "atomicity violation: cart rows changed on failed place_order. before={pre_selected_cart_rows}, after={post_selected_cart_rows}"
        ));
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_fails_on_partial_order_details_insert() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let trigger_name = format!("trg_itest_od_fail_{}", Utc::now().timestamp_micros().abs());
    create_order_details_failure_trigger(&db, &trigger_name)
        .await
        .expect("create deterministic OrderDetails failure trigger");

    let scenario_result = run_atomicity_scenario(&db, "cod").await;
    let drop_result = drop_trigger(&db, &trigger_name).await;

    if let Err(drop_err) = drop_result {
        panic!("failed to drop trigger `{trigger_name}`: {drop_err}");
    }

    if let Err(scenario_err) = scenario_result {
        panic!("{scenario_err}");
    }
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_prepaid_order_does_not_create_payment_intent_on_order_details_failure() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let trigger_name = format!(
        "trg_itest_od_fail_prepaid_{}",
        Utc::now().timestamp_micros().abs()
    );
    create_order_details_failure_trigger(&db, &trigger_name)
        .await
        .expect("create deterministic OrderDetails failure trigger");

    let scenario_result = run_atomicity_scenario(&db, "prepaid").await;
    let drop_result = drop_trigger(&db, &trigger_name).await;

    if let Err(drop_err) = drop_result {
        panic!("failed to drop trigger `{trigger_name}`: {drop_err}");
    }

    if let Err(scenario_err) = scenario_result {
        panic!("{scenario_err}");
    }
}
