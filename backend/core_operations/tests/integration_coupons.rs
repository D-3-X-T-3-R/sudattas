//! Integration tests for coupons and promotions (create, place_order with coupon, apply_coupon, limits).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_coupons -- --ignored`

mod integration_common;

use chrono::{Duration, Utc};
use integration_common::test_db_url;

use core_db_entities::entity::{
    coupons, inventory, order_status, orders, product_categories, product_variants, products,
    shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    ApplyCouponRequest, CreateCartItemRequest, CreateCouponRequest, CreateUserRequest,
    PlaceOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter,
    TransactionTrait,
};
use tonic::Request;

async fn ensure_pending_and_place_order_setup(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
    cart_total_paise: i64,
) -> (i64, i64, i64) {
    let pending = order_status::Entity::find()
        .filter(order_status::Column::StatusName.eq("pending"))
        .one(txn)
        .await
        .expect("query OrderStatus");
    if pending.is_none() {
        let status = order_status::ActiveModel {
            status_id: ActiveValue::NotSet,
            status_name: ActiveValue::Set("pending".to_string()),
        };
        let _ = status
            .insert(txn)
            .await
            .expect("insert pending OrderStatus");
    }

    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_cp_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_cp_{}", now_tag),
            email: format!("itest_cp+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user");
    let user_id = user_res.into_inner().items[0].user_id;

    let ship = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
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
    let shipping_id = ship.shipping_address_id;

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_cp_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Coupon Test Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(cart_total_paise as i32),
        category_id: ActiveValue::Set(cat.category_id),
        fabric: ActiveValue::Set(None),
        weave: ActiveValue::Set(None),
        occasion: ActiveValue::Set(None),
        length_meters: ActiveValue::Set(None),
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
        product_id: ActiveValue::Set(prod.product_id),
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
        quantity_available: ActiveValue::Set(Some(10)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(txn)
    .await
    .expect("insert Inventory");

    let _ = core_operations::handlers::cart::create_cart_item(
        txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item");

    (user_id, shipping_id, cart_total_paise)
}

/// CP1 – create_coupon + place_order with valid coupon applies discount to grand_total_minor and order snapshot.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_applied_at_checkout_reduces_total() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cart_total = 2_000_i64;
    let (user_id, shipping_id, _) =
        ensure_pending_and_place_order_setup(&txn, now_tag, cart_total).await;

    let code = format!("CP1_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 500,
            min_order_value_paise: Some(1000),
            usage_limit: Some(10),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(7)).to_rfc3339()),
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
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();
    assert_eq!(
        order.total_amount_paise, 1_500,
        "2000 - 500 discount = 1500"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(db_order.grand_total_minor, 1_500);
    assert!(db_order.applied_coupon_id.is_some());
    assert_eq!(db_order.applied_coupon_code.as_deref(), Some(code.as_str()));
    assert_eq!(db_order.applied_discount_paise, Some(500));

    txn.rollback().await.ok();
}

/// CP2 – Expired coupon (past ends_at) is ignored at checkout; full price charged and no coupon snapshot stored.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_expired_coupon_ignored_at_checkout() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cart_total = 2_000_i64;
    let (user_id, shipping_id, _) =
        ensure_pending_and_place_order_setup(&txn, now_tag, cart_total).await;

    let code = format!("CP2_EXP_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "percentage".to_string(),
            discount_value: 20,
            min_order_value_paise: Some(0),
            usage_limit: Some(100),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::days(1)).to_rfc3339(),
            ends_at: Some((Utc::now() - Duration::hours(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");

    let place_res = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: Some(code),
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();
    assert_eq!(
        order.total_amount_paise, cart_total,
        "full price when coupon expired"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(db_order.grand_total_minor, cart_total);
    assert!(db_order.applied_coupon_id.is_none());
    assert!(db_order.applied_coupon_code.is_none());

    txn.rollback().await.ok();
}

/// CP3 – apply_coupon increments usage_count for the coupon row on success.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_apply_coupon_increments_usage_count() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let create_res = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: format!("CP3_{}", now_tag),
            discount_type: "fixed_amount".to_string(),
            discount_value: 100,
            min_order_value_paise: Some(500),
            usage_limit: Some(5),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::seconds(1)).to_rfc3339(),
            ends_at: Some((Utc::now() + Duration::days(1)).to_rfc3339()),
        }),
    )
    .await
    .expect("create_coupon");
    let code = create_res.into_inner().items[0].code.clone();

    let apply_res = core_operations::handlers::coupons::apply_coupon(
        &txn,
        Request::new(ApplyCouponRequest {
            code: code.clone(),
            order_amount_paise: 1_000,
        }),
    )
    .await
    .expect("apply_coupon should succeed");
    assert!(apply_res.into_inner().items[0].is_valid);

    let coupon = coupons::Entity::find()
        .filter(coupons::Column::Code.eq(&code))
        .one(&txn)
        .await
        .expect("query coupon")
        .expect("coupon exists");
    assert_eq!(coupon.usage_count, Some(1));

    txn.rollback().await.ok();
}

/// CP4 – Coupon with usage_limit = 1: first apply_coupon succeeds, second returns invalid/limit-reached.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_usage_limit_second_apply_invalid() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let create_res = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: format!("CP4_{}", now_tag),
            discount_type: "percentage".to_string(),
            discount_value: 10,
            min_order_value_paise: Some(0),
            usage_limit: Some(1),
            max_uses_per_customer: None,
            starts_at: (Utc::now() - Duration::seconds(1)).to_rfc3339(),
            ends_at: None,
        }),
    )
    .await
    .expect("create_coupon");
    let code = create_res.into_inner().items[0].code.clone();

    let first = core_operations::handlers::coupons::apply_coupon(
        &txn,
        Request::new(ApplyCouponRequest {
            code: code.clone(),
            order_amount_paise: 1_000,
        }),
    )
    .await
    .expect("first apply_coupon should not error");
    assert!(first.into_inner().items[0].is_valid);

    let second = core_operations::handlers::coupons::apply_coupon(
        &txn,
        Request::new(ApplyCouponRequest {
            code: code.clone(),
            order_amount_paise: 1_000,
        }),
    )
    .await
    .expect("second apply_coupon returns Ok (handler does not error)");
    let result = second.into_inner().items[0].clone();
    assert!(
        !result.is_valid,
        "second apply should return is_valid false when usage limit reached"
    );

    txn.rollback().await.ok();
}

/// CP5 – Min order value not met: checkout rejects coupon and leaves order total undiscounted.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_coupon_min_order_not_met_not_applied() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let cart_total = 500_i64;
    let (user_id, shipping_id, _) =
        ensure_pending_and_place_order_setup(&txn, now_tag, cart_total).await;

    let code = format!("CP5_MIN_{}", now_tag);
    let _ = core_operations::handlers::coupons::create_coupon(
        &txn,
        Request::new(CreateCouponRequest {
            code: code.clone(),
            discount_type: "fixed_amount".to_string(),
            discount_value: 200,
            min_order_value_paise: Some(10_000),
            usage_limit: Some(10),
            max_uses_per_customer: None,
            starts_at: Utc::now().to_rfc3339(),
            ends_at: None,
        }),
    )
    .await
    .expect("create_coupon");

    let place_res = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: Some(code),
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = place_res.into_inner().items[0].clone();
    assert_eq!(
        order.total_amount_paise, cart_total,
        "total undiscounted when min not met"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(db_order.grand_total_minor, cart_total);
    assert!(db_order.applied_coupon_id.is_none());
    assert!(db_order.applied_coupon_code.is_none());

    txn.rollback().await.ok();
}
