//! Integration tests for shipping address flows and place_order using an address.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_shipping -- --ignored`

mod integration_common;

use chrono::Utc;
use integration_common::test_db_url;

use core_db_entities::entity::{
    inventory, order_status, product_categories, product_variants, products, shipping_addresses,
    user_roles,
};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    CreateCartItemRequest, CreateShippingAddressRequest, CreateUserRequest,
    DeleteShippingAddressRequest, GetShippingAddressRequest, PlaceOrderRequest,
    UpdateShippingAddressRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter,
    TransactionTrait,
};
use tonic::Request;

/// SA1 – create_shipping_address + get_shipping_address + update_shipping_address + delete_shipping_address end-to-end.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_shipping_address_crud_end_to_end() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_sa_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_sa_{}", now_tag),
            email: format!("itest_sa+{}@example.com", now_tag),
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
    .expect("create_user should succeed");
    let user_id = user_res.into_inner().items[0].user_id;

    // Create
    let create_res = core_operations::handlers::shipping_address::create_shipping_address(
        &txn,
        Request::new(CreateShippingAddressRequest {
            user_id: Some(user_id),
            country: "IN".to_string(),
            state_region: "MH".to_string(),
            city: "Mumbai".to_string(),
            postal_code: "400001".to_string(),
            road: Some("SA Road".to_string()),
            apartment_no_or_name: Some("Unit 1".to_string()),
        }),
    )
    .await
    .expect("create_shipping_address should succeed");
    let created = create_res
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("one address");
    let addr_id = created.shipping_address_id;
    assert_eq!(created.city, "Mumbai");
    assert_eq!(created.road.as_deref(), Some("SA Road"));

    // Get (returns all; find ours)
    let get_res = core_operations::handlers::shipping_address::get_shipping_address(
        &txn,
        Request::new(GetShippingAddressRequest {}),
    )
    .await
    .expect("get_shipping_address should succeed");
    let found = get_res
        .into_inner()
        .items
        .into_iter()
        .find(|a| a.shipping_address_id == addr_id)
        .expect("created address should appear in get");
    assert_eq!(found.country, "IN");
    assert_eq!(found.state_region, "MH");

    // Update
    let _ = core_operations::handlers::shipping_address::update_shipping_address(
        &txn,
        Request::new(UpdateShippingAddressRequest {
            shipping_address_id: addr_id,
            user_id: Some(user_id),
            country: "IN".to_string(),
            state_region: "KA".to_string(),
            city: "Bengaluru".to_string(),
            postal_code: "560001".to_string(),
            road: Some("Updated Road".to_string()),
            apartment_no_or_name: Some("Block B".to_string()),
        }),
    )
    .await
    .expect("update_shipping_address should succeed");

    let get_after = core_operations::handlers::shipping_address::get_shipping_address(
        &txn,
        Request::new(GetShippingAddressRequest {}),
    )
    .await
    .expect("get_shipping_address should succeed");
    let updated = get_after
        .into_inner()
        .items
        .into_iter()
        .find(|a| a.shipping_address_id == addr_id)
        .expect("address should still exist after update");
    assert_eq!(updated.city, "Bengaluru");
    assert_eq!(updated.road.as_deref(), Some("Updated Road"));
    assert_eq!(updated.state_region, "KA");

    // Delete
    let _ = core_operations::handlers::shipping_address::delete_shipping_address(
        &txn,
        Request::new(DeleteShippingAddressRequest {
            shipping_address_id: addr_id,
        }),
    )
    .await
    .expect("delete_shipping_address should succeed");

    let get_final = core_operations::handlers::shipping_address::get_shipping_address(
        &txn,
        Request::new(GetShippingAddressRequest {}),
    )
    .await
    .expect("get_shipping_address should succeed");
    let gone = get_final
        .into_inner()
        .items
        .into_iter()
        .any(|a| a.shipping_address_id == addr_id);
    assert!(!gone, "deleted address should not appear in get");

    txn.rollback().await.ok();
}

/// SA2 – place_order uses the expected shipping address; order row matches address fields (shipping_address_id).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_uses_expected_shipping_address() {
    use core_db_entities::entity::orders;

    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

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

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_sa2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_sa2_{}", now_tag),
            email: format!("itest_sa2+{}@example.com", now_tag),
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
    .expect("create_user should succeed");
    let user_id = user_res.into_inner().items[0].user_id;

    let create_addr = core_operations::handlers::shipping_address::create_shipping_address(
        &txn,
        Request::new(CreateShippingAddressRequest {
            user_id: Some(user_id),
            country: "IN".to_string(),
            state_region: "TN".to_string(),
            city: "Chennai".to_string(),
            postal_code: "600001".to_string(),
            road: Some("Order Address Road".to_string()),
            apartment_no_or_name: Some("Flat 5".to_string()),
        }),
    )
    .await
    .expect("create_shipping_address should succeed");
    let shipping_id = create_addr.into_inner().items[0].shipping_address_id;

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_sa2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("SA2 Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(500),
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
    .expect("insert Products");

    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
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
        quantity_available: ActiveValue::Set(Some(5)),
        quantity_reserved: ActiveValue::Set(Some(0)),
        reorder_level: ActiveValue::Set(None),
        updated_at: ActiveValue::Set(Some(Utc::now())),
    }
    .insert(&txn)
    .await
    .expect("insert Inventory");

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
    .expect("create_cart_item should succeed");

    let place_res = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping_id,
            user_id,
            coupon_code: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let order = &place_res.into_inner().items[0];
    assert_eq!(
        order.shipping_address_id, shipping_id,
        "order should reference the created shipping address"
    );

    let db_order = orders::Entity::find_by_id(order.order_id)
        .one(&txn)
        .await
        .expect("query order")
        .expect("order exists");
    assert_eq!(
        db_order.shipping_address_id, shipping_id,
        "order row should match shipping_address_id"
    );

    let addr = shipping_addresses::Entity::find_by_id(shipping_id)
        .one(&txn)
        .await
        .expect("query address")
        .expect("address exists");
    assert_eq!(addr.city, "Chennai");
    assert_eq!(addr.postal_code, "600001");

    txn.rollback().await.ok();
}
