//! Integration tests for order state machine (update_order, admin_mark_shipped/delivered, inventory restore).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_order_state -- --ignored`

mod integration_common;

use chrono::Utc;
use integration_common::test_db_url;

use core_db_entities::entity::{
    inventory, order_events, order_status, orders, product_categories, product_variants, products,
    shipments, shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    AdminMarkOrderDeliveredRequest, AdminMarkOrderShippedRequest, CreateCartItemRequest,
    CreateUserRequest, PlaceOrderRequest, UpdateOrderRequest,
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

/// Build user + shipping + one product/variant/inventory + one cart item, place order; return (order_id, user_id, shipping_id, variant_id, total_paise).
async fn place_order_minimal(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
) -> (i64, i64, i64, i64, i64) {
    let _ = ensure_order_status(txn, "pending").await;
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_ord_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_ord_{}", now_tag),
            email: format!("itest_ord+{}@example.com", now_tag),
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
        name: ActiveValue::Set(format!("itest_cat_ord_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let prod = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Order State Product".to_string()),
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
        variant.variant_id,
        order.total_amount_paise,
    )
}

/// O1 – update_order transitions pending → confirmed; order row updated and order_events entry created.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_order_update_pending_to_confirmed_and_order_event() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, shipping_id, _variant_id, total_paise) =
        place_order_minimal(&txn, now_tag).await;

    let confirmed_id = ensure_order_status(&txn, "confirmed").await;

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
    .expect("update_order to confirmed should succeed");

    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, confirmed_id);

    let events = order_events::Entity::find()
        .filter(order_events::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query order_events");
    assert!(
        events
            .iter()
            .any(|e| e.to_status.as_deref() == Some("confirmed")),
        "order_events should contain transition to confirmed"
    );

    txn.rollback().await.ok();
}

/// O2 – Cancelling an order via update_order restores inventory quantities from order_details.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_order_cancel_restores_inventory() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, shipping_id, variant_id, total_paise) =
        place_order_minimal(&txn, now_tag).await;

    let inv_before = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(Some(variant_id)))
        .one(&txn)
        .await
        .expect("query inventory")
        .expect("inventory exists");
    let qty_before = inv_before.quantity_available.unwrap_or(0);

    let cancelled_id = ensure_order_status(&txn, "cancelled").await;

    let _ = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: cancelled_id,
        }),
    )
    .await
    .expect("update_order to cancelled should succeed");

    let inv_after = inventory::Entity::find()
        .filter(inventory::Column::VariantId.eq(Some(variant_id)))
        .one(&txn)
        .await
        .expect("query inventory")
        .expect("inventory exists");
    let qty_after = inv_after.quantity_available.unwrap_or(0);
    assert_eq!(
        qty_after,
        qty_before + 1,
        "inventory should be restored by 1 (one line item quantity)"
    );

    txn.rollback().await.ok();
}

/// O3 – Illegal transition (pending → delivered) via update_order returns InvalidArgument.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_order_illegal_transition_returns_invalid_argument() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, shipping_id, _variant_id, total_paise) =
        place_order_minimal(&txn, now_tag).await;

    let delivered_id = ensure_order_status(&txn, "delivered").await;

    let result = core_operations::handlers::orders::update_order(
        &txn,
        Request::new(UpdateOrderRequest {
            order_id,
            user_id,
            shipping_address_id: shipping_id,
            total_amount_paise: total_paise,
            status_id: delivered_id,
        }),
    )
    .await;

    let err = result.expect_err("update_order pending→delivered should fail");
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(
        err.message().to_lowercase().contains("illegal") || err.message().contains("transition"),
        "error should mention illegal transition, got: {}",
        err.message()
    );

    txn.rollback().await.ok();
}

/// O4 – admin_mark_order_shipped transitions order to shipped and creates a shipment row when tracking is provided.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_admin_mark_shipped_creates_shipment() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, shipping_id, _variant_id, total_paise) =
        place_order_minimal(&txn, now_tag).await;

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

    let ship_res = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("AWB123".to_string()),
            carrier: Some("DHL".to_string()),
        }),
    )
    .await
    .expect("admin_mark_order_shipped should succeed");
    assert!(
        ship_res.into_inner().shipment_id > 0,
        "shipment should be created"
    );

    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    let shipped_id = ensure_order_status(&txn, "shipped").await;
    assert_eq!(order.status_id, shipped_id);

    let ship_rows = shipments::Entity::find()
        .filter(shipments::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query Shipments");
    assert_eq!(ship_rows.len(), 1);
    assert_eq!(ship_rows[0].awb_code.as_deref(), Some("AWB123"));
    assert_eq!(ship_rows[0].carrier.as_deref(), Some("DHL"));

    txn.rollback().await.ok();
}

/// O5 – admin_mark_order_shipped called twice updates existing shipment (awb/carrier) instead of creating a new one.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_admin_mark_shipped_twice_updates_shipment() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, shipping_id, _variant_id, total_paise) =
        place_order_minimal(&txn, now_tag).await;

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
            awb_code: Some("AWB1".to_string()),
            carrier: Some("Carrier1".to_string()),
        }),
    )
    .await
    .expect("first admin_mark_order_shipped");

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("AWB2".to_string()),
            carrier: Some("Carrier2".to_string()),
        }),
    )
    .await
    .expect("second admin_mark_order_shipped");

    let ship_rows = shipments::Entity::find()
        .filter(shipments::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query Shipments");
    assert_eq!(ship_rows.len(), 1, "should still be one shipment row");
    assert_eq!(ship_rows[0].awb_code.as_deref(), Some("AWB2"));
    assert_eq!(ship_rows[0].carrier.as_deref(), Some("Carrier2"));

    txn.rollback().await.ok();
}

/// O6 – admin_mark_order_delivered transitions shipped → delivered and records the change.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_admin_mark_delivered_transitions_to_delivered() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, shipping_id, _variant_id, total_paise) =
        place_order_minimal(&txn, now_tag).await;

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
            awb_code: Some("AWB".to_string()),
            carrier: Some("Carrier".to_string()),
        }),
    )
    .await
    .expect("mark shipped");

    let _ = core_operations::handlers::orders::admin_mark_order_delivered(
        &txn,
        Request::new(AdminMarkOrderDeliveredRequest { order_id }),
    )
    .await
    .expect("admin_mark_order_delivered should succeed");

    let order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    let delivered_id = ensure_order_status(&txn, "delivered").await;
    assert_eq!(order.status_id, delivered_id);

    txn.rollback().await.ok();
}

/// O7 – Full lifecycle: pending → confirmed → processing → shipped → delivered using allowed transitions only.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_order_full_lifecycle_pending_to_delivered() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (order_id, user_id, shipping_id, _variant_id, total_paise) =
        place_order_minimal(&txn, now_tag).await;

    let pending_id = ensure_order_status(&txn, "pending").await;
    let confirmed_id = ensure_order_status(&txn, "confirmed").await;
    let processing_id = ensure_order_status(&txn, "processing").await;
    let _shipped_id = ensure_order_status(&txn, "shipped").await;
    let delivered_id = ensure_order_status(&txn, "delivered").await;

    let mut order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, pending_id);

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
    .expect("pending → confirmed");

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
    .expect("confirmed → processing");

    let _ = core_operations::handlers::orders::admin_mark_order_shipped(
        &txn,
        Request::new(AdminMarkOrderShippedRequest {
            order_id,
            awb_code: Some("FULL-AWB".to_string()),
            carrier: Some("FullCarrier".to_string()),
        }),
    )
    .await
    .expect("processing → shipped");

    let _ = core_operations::handlers::orders::admin_mark_order_delivered(
        &txn,
        Request::new(AdminMarkOrderDeliveredRequest { order_id }),
    )
    .await
    .expect("shipped → delivered");

    order = orders::Entity::find_by_id(order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(order.status_id, delivered_id);

    let events = order_events::Entity::find()
        .filter(order_events::Column::OrderId.eq(order_id))
        .all(&txn)
        .await
        .expect("query order_events");
    let to_statuses: Vec<Option<&str>> = events.iter().map(|e| e.to_status.as_deref()).collect();
    assert!(to_statuses.contains(&Some("confirmed")));
    assert!(to_statuses.contains(&Some("processing")));
    assert!(to_statuses.contains(&Some("shipped")));
    assert!(to_statuses.contains(&Some("delivered")));

    txn.rollback().await.ok();
}
