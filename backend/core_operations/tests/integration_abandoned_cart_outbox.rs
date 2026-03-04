//! Integration tests for abandoned cart (enqueue_abandoned_cart_events) and outbox (place_order, shipped, delivered).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_abandoned_cart_outbox -- --ignored`
//!
//! **Note:** AC1 and AC2 commit data (enqueue_abandoned_cart_events commits); OB1 and OB2 use rollback.

mod integration_common;

use chrono::{Duration, Utc};
use core_db_entities::entity::{
    inventory, order_status, outbox_events, product_categories, product_variants, products,
    shipping_addresses, user_roles, users,
};
use core_operations::handlers::outbox::{DELIVERED, ORDER_PLACED, SHIPPED};
use core_operations::order_state_machine;
use core_operations::procedures::abandoned_cart::enqueue_abandoned_cart_events;
use core_operations::procedures::orders::place_order;
use integration_common::test_db_url;
use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, CreateCartItemRequest,
    CreateUserRequest, PlaceOrderRequest, UpdateOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, EntityTrait,
    IntoActiveModel, QueryFilter, Statement, TransactionTrait,
};
use tonic::Request;

async fn ensure_order_status(txn: &sea_orm::DatabaseTransaction, name: &str) -> i64 {
    if let Ok(Some(id)) = order_state_machine::get_status_id(txn, name).await {
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

/// AC1 – Stale user cart with marketing_opt_out = 0 triggers enqueue_abandoned_cart_events and enqueues one outbox event.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_abandoned_cart_opt_in_enqueues_one_event() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let txn = db.begin().await.expect("begin");
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ac1_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ac1_{}", now_tag),
            email: format!("itest_ac1+{}@example.com", now_tag),
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

    let _ = txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            "UPDATE Users SET marketing_opt_out = 0 WHERE UserID = ?",
            [user_id.into()],
        ))
        .await
        .expect("set marketing_opt_out = 0");

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_ac1_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("AC Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(1_000),
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

    let _ = core_operations::handlers::cart::create_cart_item(
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

    let stale_at = Utc::now() - Duration::hours(25);
    let _ = txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            "UPDATE Cart SET updated_at = ? WHERE UserID = ?",
            [stale_at.into(), user_id.into()],
        ))
        .await
        .expect("set cart updated_at to stale");

    txn.commit().await.expect("commit setup");

    let count = enqueue_abandoned_cart_events(&db, 24)
        .await
        .expect("enqueue_abandoned_cart_events");
    assert!(
        count >= 1,
        "at least one user with stale cart and opt-in should enqueue (count may be >1 from prior runs)"
    );

    let txn2 = db.begin().await.expect("begin");
    let events = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq("AbandonedCart"))
        .filter(outbox_events::Column::AggregateId.eq(user_id.to_string()))
        .all(&txn2)
        .await
        .expect("query outbox_events");
    txn2.rollback().await.ok();
    assert!(
        !events.is_empty(),
        "our user should have at least one AbandonedCart event (may be >1 from prior runs)"
    );
}

/// AC2 – Stale cart but marketing_opt_out = 1 results in no abandoned-cart events enqueued.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_abandoned_cart_opt_out_no_events() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");

    let now_tag = Utc::now().timestamp_millis();
    let txn = db.begin().await.expect("begin");
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ac2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ac2_{}", now_tag),
            email: format!("itest_ac2+{}@example.com", now_tag),
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

    let user = users::Entity::find_by_id(user_id)
        .one(&txn)
        .await
        .expect("find user")
        .expect("user exists");
    let mut active = user.into_active_model();
    active.marketing_opt_out = ActiveValue::Set(Some(1));
    active
        .update(&txn)
        .await
        .expect("set marketing_opt_out = 1");

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_ac2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("AC2 Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(1_000),
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

    let _ = core_operations::handlers::cart::create_cart_item(
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

    let stale_at = Utc::now() - Duration::hours(25);
    let _ = txn
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::MySql,
            "UPDATE Cart SET updated_at = ? WHERE UserID = ?",
            [stale_at.into(), user_id.into()],
        ))
        .await
        .expect("set cart updated_at to stale");

    txn.commit().await.expect("commit setup");

    let _count = enqueue_abandoned_cart_events(&db, 24)
        .await
        .expect("enqueue_abandoned_cart_events");

    let txn2 = db.begin().await.expect("begin");
    let events = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq("AbandonedCart"))
        .filter(outbox_events::Column::AggregateId.eq(user_id.to_string()))
        .all(&txn2)
        .await
        .expect("query outbox_events");
    txn2.rollback().await.ok();
    assert!(
        events.is_empty(),
        "opted-out user should have no AbandonedCart event (found {}); total enqueued may be >0 from other users",
        events.len()
    );
}

/// Place order minimal setup; return (order_id, user_id, shipping_id, total_paise).
async fn place_order_setup(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
) -> (i64, i64, i64, i64) {
    let _ = ensure_order_status(txn, "pending").await;
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ob_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ob_{}", now_tag),
            email: format!("itest_ob+{}@example.com", now_tag),
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
        name: ActiveValue::Set(format!("itest_cat_ob_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Outbox Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(2_000),
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

    let place_res = place_order(
        txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
        }),
    )
    .await
    .expect("place_order");
    let order = place_res.into_inner().items[0].clone();
    (
        order.order_id,
        user_id,
        shipping_id,
        order.total_amount_paise,
    )
}

/// OB1 – place_order enqueues an OrderPlaced outbox event with correct payload.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_enqueues_order_placed_outbox() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, _shipping_id, _total_paise) = place_order_setup(&txn, now_tag).await;

    let events = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(ORDER_PLACED))
        .filter(outbox_events::Column::AggregateId.eq(order_id.to_string()))
        .all(&txn)
        .await
        .expect("query outbox_events");
    assert_eq!(events.len(), 1);
    let payload = &events[0].payload;
    assert_eq!(
        payload
            .get("order_id")
            .and_then(|v: &serde_json::Value| v.as_i64()),
        Some(order_id)
    );
    assert_eq!(
        payload
            .get("user_id")
            .and_then(|v: &serde_json::Value| v.as_i64()),
        Some(user_id)
    );

    txn.rollback().await.ok();
}

/// OB2 – admin_mark_order_shipped enqueues Shipped; admin_mark_order_delivered enqueues Delivered.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_shipped_delivered_enqueue_outbox_events() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, shipping_id, total_paise) = place_order_setup(&txn, now_tag).await;

    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let processing_id = ensure_order_status(&txn, "processing").await;

    let _ = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: confirmed_id,
        }),
    )
    .await
    .expect("update to confirmed");

    let _ = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: processing_id,
        }),
    )
    .await
    .expect("update to processing");

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("OB2AWB".to_string()),
            carrier: Some("Carrier".to_string()),
        }),
    )
    .await
    .expect("admin_mark_order_shipped");

    let shipped = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(SHIPPED))
        .filter(outbox_events::Column::AggregateId.eq(order_id.to_string()))
        .all(&txn)
        .await
        .expect("query outbox_events");
    assert_eq!(shipped.len(), 1);
    assert_eq!(
        shipped[0]
            .payload
            .get("order_id")
            .and_then(|v: &serde_json::Value| v.as_i64()),
        Some(order_id)
    );

    let _ = core_operations::handlers::orders::admin_mark_order_delivered(
        &txn,
        Request::new(AdminMarkOrderDeliveredRequest { order_id }),
    )
    .await
    .expect("admin_mark_order_delivered");

    let delivered = outbox_events::Entity::find()
        .filter(outbox_events::Column::EventType.eq(DELIVERED))
        .filter(outbox_events::Column::AggregateId.eq(order_id.to_string()))
        .all(&txn)
        .await
        .expect("query outbox_events");
    assert_eq!(delivered.len(), 1);
    assert_eq!(
        delivered[0]
            .payload
            .get("order_id")
            .and_then(|v: &serde_json::Value| v.as_i64()),
        Some(order_id)
    );

    txn.rollback().await.ok();
}
