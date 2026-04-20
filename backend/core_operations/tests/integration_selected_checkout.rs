//! Integration tests for selected-cart-line checkout.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_selected_checkout -- --ignored`

mod integration_common;

use chrono::{Duration, Utc};
use integration_common::test_db_url;

use core_db_entities::entity::{
    cart, coupons, inventory, order_details, order_status, orders, payment_intents,
    product_categories, product_variants, products, shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    CreateCartItemRequest, CreateCouponRequest, CreateUserRequest, PlaceOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter,
    TransactionTrait,
};
use tonic::{Code, Request};

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

async fn create_checkout_user(txn: &sea_orm::DatabaseTransaction, now_tag: i64) -> (i64, i64) {
    let _ = ensure_order_status(txn, "pending").await;

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_sel_role_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_sel_{}", now_tag),
            email: format!("itest_sel+{}@example.com", now_tag),
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

    let shipping = shipping_addresses::ActiveModel {
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
    .insert(txn)
    .await
    .expect("insert ShippingAddresses");

    (user_id, shipping.shipping_address_id)
}

async fn create_variant_with_cart_item(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
    user_id: i64,
    name_suffix: &str,
    price_paise: i32,
    inventory_qty: i32,
    cart_qty: i64,
) -> (i64, i64) {
    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_sel_cat_{}_{}", name_suffix, now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set(format!("Selected Checkout {}", name_suffix)),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(price_paise),
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
    .insert(txn)
    .await
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(txn)
    .await
    .expect("insert ProductVariants");

    let _ = inventory::ActiveModel {
        inventory_id: ActiveValue::NotSet,
        variant_id: ActiveValue::Set(Some(variant.variant_id)),
        quantity_available: ActiveValue::Set(Some(i64::from(inventory_qty))),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert Inventory");

    let cart_res = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: cart_qty,
        }),
    )
    .await
    .expect("create_cart_item");

    (variant.variant_id, cart_res.into_inner().items[0].cart_id)
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_subset_creates_order_for_only_selected_lines() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id) = create_checkout_user(&txn, now_tag).await;
    let (selected_variant_id, selected_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "A", 1_000, 10, 2).await;
    let (unselected_variant_id, unselected_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "B", 2_000, 10, 1).await;

    let place_res = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();

    assert_eq!(
        order.total_amount_paise, 2_000,
        "only the selected 2 x 1000 line is charged"
    );

    let detail_rows = order_details::Entity::find()
        .filter(order_details::Column::OrderId.eq(order.order_id))
        .all(&txn)
        .await
        .expect("query order_details");
    assert_eq!(
        detail_rows.len(),
        1,
        "only selected line should become an order detail"
    );
    assert_eq!(detail_rows[0].variant_id, selected_variant_id);
    assert_eq!(detail_rows[0].quantity, 2);

    let payment_intent = payment_intents::Entity::find()
        .filter(payment_intents::Column::OrderId.eq(order.order_id))
        .one(&txn)
        .await
        .expect("query payment_intents")
        .expect("payment intent exists");
    assert_eq!(payment_intent.amount_paise, 2_000);

    let remaining_cart = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&txn)
        .await
        .expect("query cart");
    assert_eq!(
        remaining_cart.len(),
        1,
        "unselected line should remain in cart"
    );
    assert_eq!(remaining_cart[0].cart_id, unselected_cart_id);
    assert_eq!(remaining_cart[0].variant_id, unselected_variant_id);

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_subset_coupon_logic_uses_only_selected_lines() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id) = create_checkout_user(&txn, now_tag).await;
    let (_selected_variant_id, selected_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "CouponA", 600, 10, 1).await;
    let _ = create_variant_with_cart_item(&txn, now_tag, user_id, "CouponB", 600, 10, 1).await;

    let code = format!("SELCP_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 500,
            min_order_value_paise: Some(1_000),
            usage_limit: Some(10),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");

    let place_res = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: Some(code.clone()),
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();

    assert_eq!(
        order.total_amount_paise, 600,
        "coupon should not apply because selected subset is below min order"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert!(
        db_order.applied_coupon_id.is_none(),
        "coupon snapshot should not be stored"
    );
    assert!(db_order.applied_coupon_code.is_none());

    let coupon_row = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn)
        .await
        .expect("query coupon")
        .expect("coupon exists");
    assert_eq!(
        coupon_row.usage_count,
        Some(0),
        "usage should not increment before payment or on non-applicable subset"
    );

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_checkout_rejects_empty_or_invalid_selection() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id) = create_checkout_user(&txn, now_tag).await;
    let (_variant_id, cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "Invalid", 900, 10, 1).await;

    let empty = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![],
            payment_mode: None,
        }),
    )
    .await
    .expect_err("empty selection should fail");
    assert_eq!(empty.code(), Code::FailedPrecondition);
    assert!(empty.message().to_lowercase().contains("selected"));

    let invalid = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![cart_id, cart_id + 999_999],
            payment_mode: None,
        }),
    )
    .await
    .expect_err("invalid selection should fail");
    assert_eq!(invalid.code(), Code::InvalidArgument);
    assert!(invalid
        .message()
        .to_lowercase()
        .contains("selected cart item"));

    let remaining_cart = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&txn)
        .await
        .expect("query cart");
    assert_eq!(
        remaining_cart.len(),
        1,
        "cart should remain untouched on selection validation failure"
    );

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_selected_checkout_rejects_out_of_stock_selected_line() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, shipping_id) = create_checkout_user(&txn, now_tag).await;
    let (_variant_id, selected_cart_id) =
        create_variant_with_cart_item(&txn, now_tag, user_id, "LowStock", 1_200, 1, 2).await;
    let _ = create_variant_with_cart_item(&txn, now_tag, user_id, "Safe", 800, 10, 1).await;

    let result = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![selected_cart_id],
            payment_mode: None,
        }),
    )
    .await
    .expect_err("out-of-stock selected line should fail");

    assert_eq!(result.code(), Code::FailedPrecondition);
    assert!(result.message().contains("Insufficient stock for variant"));

    let order_count = orders::Entity::find()
        .filter(orders::Column::UserId.eq(user_id))
        .all(&txn)
        .await
        .expect("query orders")
        .len();
    assert_eq!(order_count, 0, "no order should be created");

    let remaining_cart = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&txn)
        .await
        .expect("query cart");
    assert_eq!(
        remaining_cart.len(),
        2,
        "all cart rows should remain for retry"
    );

    txn.rollback().await.ok();
}
