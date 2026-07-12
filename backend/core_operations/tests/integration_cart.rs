//! Integration tests for cart behavior (delete one item, guest vs user, clear after payment / Paid).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_cart -- --ignored`

mod integration_common;
mod provider_test_gate;

use chrono::Utc;
use integration_common::test_db_url;

use core_db_entities::entity::{
    cart, inventory, order_status, product_categories, product_variants, products,
    shipping_addresses, user_roles,
};
use core_operations::procedures::orders::place_order;
use proto::proto::core::{
    CreateCartItemRequest, CreateUserRequest, DeleteCartItemRequest, GetCartItemsRequest,
    PlaceOrderRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter,
    TransactionTrait,
};
use tonic::{Code, Request};

#[allow(dead_code)]
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

/// C1 – create_cart_item × 2 then delete_cart_item with cart_id leaves the other item intact and returned.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_cart_delete_one_item_returns_remaining() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_c1_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_c1_{}", now_tag),
            email: format!("itest_c1+{}@example.com", now_tag),
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

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_c1_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("C1 Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        // Keep selected subtotal above FREE_SHIPPING_THRESHOLD_MINOR to avoid live quote dependency.
        price_paise: ActiveValue::Set(150_000),
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

    let variant_a = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");
    let variant_b = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    let cart1 = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant_a.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item 1");
    let cart_id_a = cart1.into_inner().items[0].cart_id;

    let cart2 = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant_b.variant_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create_cart_item 2");
    let cart_id_b = cart2.into_inner().items[0].cart_id;

    let delete_res = core_operations::handlers::cart::delete_cart_item(
        &txn,
        Request::new(DeleteCartItemRequest {
            user_id: Some(user_id),
            cart_id: Some(cart_id_a),
            session_id: None,
        }),
    )
    .await
    .expect("delete_cart_item should succeed");
    let remaining = delete_res.into_inner().items;
    assert_eq!(remaining.len(), 1, "one item should remain");
    assert_eq!(remaining[0].cart_id, cart_id_b);
    assert_eq!(remaining[0].variant_id, variant_b.variant_id);
    assert_eq!(remaining[0].quantity, 2);

    let get_res = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("get_cart_items should succeed");
    assert_eq!(get_res.get_ref().items.len(), 1);
    assert_eq!(get_res.get_ref().items[0].cart_id, cart_id_b);

    txn.rollback().await.ok();
}

/// C1b – create_cart_item called twice for the same (user, variant) increments quantity
/// on a single row instead of erroring on the unique constraint.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_cart_item_duplicate_variant_increments_quantity() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_c1b_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_c1b_{}", now_tag),
            email: format!("itest_c1b+{}@example.com", now_tag),
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

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_c1b_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("C1b Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(150_000),
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

    let first = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item (first add) should succeed");
    let cart_id = first.into_inner().items[0].cart_id;

    let second = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant.variant_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create_cart_item (repeat add) should succeed, not error as a duplicate");
    let second_items = second.into_inner().items;
    assert_eq!(
        second_items.len(),
        1,
        "repeat add should return exactly one (incremented) row"
    );
    assert_eq!(
        second_items[0].cart_id, cart_id,
        "repeat add should update the same cart row, not create a new one"
    );
    assert_eq!(
        second_items[0].quantity, 3,
        "quantity should be summed (1 + 2), not overwritten or duplicated"
    );

    let get_res = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("get_cart_items should succeed");
    assert_eq!(
        get_res.get_ref().items.len(),
        1,
        "only one cart row should exist for this (user, variant) pair"
    );
    assert_eq!(get_res.get_ref().items[0].quantity, 3);

    txn.rollback().await.ok();
}

/// C1c – merge_cart moves guest (session-scoped) cart rows into the user's cart:
/// a variant only in the guest cart is reassigned; a variant present in both
/// carts has its quantities summed into the user's row, and the guest row is
/// dropped. The guest session's cart is empty afterward.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_merge_cart_moves_guest_items_and_sums_overlap() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let session_id = format!("itest_guest_session_{}", now_tag);
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_c1c_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_c1c_{}", now_tag),
            email: format!("itest_c1c+{}@example.com", now_tag),
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

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_c1c_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("C1c Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(150_000),
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

    // Guest-only variant: present only in the guest cart before merge.
    let variant_guest_only = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants (guest-only)");

    // Overlapping variant: present in both the guest cart and the user's cart.
    let variant_overlap = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants (overlap)");

    // Guest cart: 2x guest-only variant, 3x overlap variant.
    core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: None,
            session_id: Some(session_id.clone()),
            variant_id: variant_guest_only.variant_id,
            quantity: 2,
        }),
    )
    .await
    .expect("create_cart_item (guest, guest-only variant)");
    core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: None,
            session_id: Some(session_id.clone()),
            variant_id: variant_overlap.variant_id,
            quantity: 3,
        }),
    )
    .await
    .expect("create_cart_item (guest, overlap variant)");

    // User already has 1x the overlap variant before merging.
    core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: variant_overlap.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item (user, overlap variant)");

    let merge_res = core_operations::handlers::cart::merge_cart(
        &txn,
        Request::new(proto::proto::core::MergeCartRequest {
            user_id,
            session_id: session_id.clone(),
        }),
    )
    .await
    .expect("merge_cart should succeed")
    .into_inner()
    .items;

    assert_eq!(
        merge_res.len(),
        2,
        "user should have exactly 2 cart rows after merge: {:?}",
        merge_res
    );
    let guest_only_row = merge_res
        .iter()
        .find(|item| item.variant_id == variant_guest_only.variant_id)
        .expect("guest-only variant should be reassigned to the user");
    assert_eq!(guest_only_row.quantity, 2);
    assert_eq!(guest_only_row.user_id, user_id);
    let overlap_row = merge_res
        .iter()
        .find(|item| item.variant_id == variant_overlap.variant_id)
        .expect("overlap variant should exist in the user's cart");
    assert_eq!(
        overlap_row.quantity, 4,
        "overlap variant quantity should be summed (1 existing + 3 from guest)"
    );

    let guest_cart_after = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(GetCartItemsRequest {
            user_id: None,
            session_id: Some(session_id.clone()),
        }),
    )
    .await
    .expect("get_cart_items (guest) should succeed");
    assert!(
        guest_cart_after.get_ref().items.is_empty(),
        "guest session cart should be empty after merge"
    );

    txn.rollback().await.ok();
}

/// C2 – Guest cart (session_id only) add + get_cart_items; verify place_order cannot proceed without a user_id (user's cart is empty).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_guest_cart_not_used_for_place_order() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_c2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_c2_{}", now_tag),
            email: format!("itest_c2+{}@example.com", now_tag),
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

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_c2_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("C2 Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(100),
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

    let session_id = format!("guest-session-{}", now_tag);
    let guest_cart = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: None,
            session_id: Some(session_id.clone()),
            variant_id: variant.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item (guest) should succeed");
    let guest_cart_id = guest_cart.into_inner().items[0].cart_id;

    let get_res = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(GetCartItemsRequest {
            user_id: None,
            session_id: Some(session_id.clone()),
        }),
    )
    .await
    .expect("get_cart_items by session_id should succeed");
    assert_eq!(get_res.get_ref().items.len(), 1, "guest cart has one item");

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
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");

    let result = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: vec![guest_cart_id],
            payment_mode: None,
        }),
    )
    .await;
    let err = result.expect_err(
        "place_order should fail because selected guest cart ids do not belong to the customer cart",
    );
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(
        err.message().to_lowercase().contains("selected cart item")
            && err.message().to_lowercase().contains("not found"),
        "error should mention invalid selected ownership, got: {}",
        err.message()
    );

    txn.rollback().await.ok();
}

/// C3 – place_order removes only selected user cart items and leaves unselected rows intact.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_place_order_removes_only_selected_user_cart_items() {
    if !provider_test_gate::should_run_provider_dependent_test(
        "integration_place_order_removes_only_selected_user_cart_items",
    ) {
        return;
    }

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
        role_name: ActiveValue::Set(format!("itest_role_c3_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_c3_{}", now_tag),
            email: format!("itest_c3+{}@example.com", now_tag),
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
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_c3_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("C3 Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        // Keep selected subtotal above FREE_SHIPPING_THRESHOLD_MINOR to avoid live quote dependency.
        price_paise: ActiveValue::Set(150_000),
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

    let v1 = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");
    let v2 = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");
    let v3 = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product.product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(Some(0)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");

    for (vid, _) in [
        (v1.variant_id, 1_i64),
        (v2.variant_id, 1),
        (v3.variant_id, 1),
    ] {
        let _ = inventory::ActiveModel {
            inventory_id: ActiveValue::NotSet,
            variant_id: ActiveValue::Set(Some(vid)),
            quantity_available: ActiveValue::Set(Some(10)),
            quantity_reserved: ActiveValue::Set(Some(0)),
            reorder_level: ActiveValue::Set(None),
            updated_at: ActiveValue::Set(Some(Utc::now())),
        }
        .insert(&txn)
        .await
        .expect("insert Inventory");
    }

    let cart_1 = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: v1.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item 1");
    let cart_2 = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: v2.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item 2");
    let cart_3 = core_operations::handlers::cart::create_cart_item(
        &txn,
        Request::new(CreateCartItemRequest {
            user_id: Some(user_id),
            session_id: None,
            variant_id: v3.variant_id,
            quantity: 1,
        }),
    )
    .await
    .expect("create_cart_item 3");
    let selected_cart_ids = vec![
        cart_1.into_inner().items[0].cart_id,
        cart_3.into_inner().items[0].cart_id,
    ];
    let unselected_cart_id = cart_2.into_inner().items[0].cart_id;

    let get_before = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("get_cart_items");
    assert_eq!(
        get_before.get_ref().items.len(),
        3,
        "three items before place_order"
    );

    let place_res = place_order(
        &txn,
        Request::new(PlaceOrderRequest {
            shipping_address_id: shipping.shipping_address_id,
            user_id,
            coupon_code: None,
            selected_cart_ids: selected_cart_ids.clone(),
            payment_mode: None,
        }),
    )
    .await
    .expect("place_order should succeed");
    let place_body = place_res.into_inner();
    assert_eq!(place_body.items.len(), 1);
    let _order_id = place_body.items[0].order_id;

    let get_after_place = core_operations::handlers::cart::get_cart_items(
        &txn,
        Request::new(GetCartItemsRequest {
            user_id: Some(user_id),
            session_id: None,
        }),
    )
    .await
    .expect("get_cart_items");
    assert_eq!(
        get_after_place.get_ref().items.len(),
        1,
        "only unselected cart rows should remain after place_order"
    );
    assert_eq!(
        get_after_place.get_ref().items[0].cart_id,
        unselected_cart_id
    );

    let cart_rows = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&txn)
        .await
        .expect("query Cart");
    assert_eq!(cart_rows.len(), 1, "one unselected cart row should remain");
    assert_eq!(cart_rows[0].cart_id, unselected_cart_id);

    txn.rollback().await.ok();
}
