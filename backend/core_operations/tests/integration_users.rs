//! Integration tests for user lifecycle flows (create, update, delete, PII export).
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL` (e.g. `mysql://root:test_password@127.0.0.1:3306/sudattas_test`).
//! - Schema must be loaded first (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_users -- --ignored`

mod integration_common;

use chrono::Utc;
use integration_common::test_db_url;

use core_db_entities::entity::{cart, product_categories, product_variants, products, user_roles};
use proto::proto::core::{
    CreateCartItemRequest, CreateUserRequest, DeleteUserRequest, GetUserPiiExportRequest,
    SearchUserRequest, UpdateUserRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, EntityTrait, QueryFilter,
    TransactionTrait,
};
use tonic::{Code, Request};

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_user_create_and_search() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let role_name = format!("itest_role_search_{}", Utc::now().timestamp_millis());
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(role_name),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let now_tag = Utc::now().timestamp_millis();
    let username = format!("itest_search_user_{}", now_tag);
    let email = format!("itest_search+{}@example.com", now_tag);
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: username.clone(),
            email: email.clone(),
            full_name: Some("Search Test User".to_string()),
            address: Some("789 Search St".to_string()),
            phone: Some("5551234567".to_string()),
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user should succeed");
    let created = user_res
        .into_inner()
        .items
        .into_iter()
        .next()
        .expect("one user");
    let user_id = created.user_id;

    let search_res = core_operations::handlers::users::search_user(
        &txn,
        Request::new(SearchUserRequest { user_id }),
    )
    .await
    .expect("search_user should succeed");
    let found = search_res.into_inner().items;
    assert_eq!(found.len(), 1, "search by user_id should return one user");
    assert_eq!(found[0].user_id, user_id);
    assert_eq!(found[0].username, username);
    assert_eq!(found[0].email, email);

    txn.rollback().await.ok();
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_user_create_update_search() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let role_name = format!("itest_role_upd_{}", Utc::now().timestamp_millis());
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(role_name),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let now_tag = Utc::now().timestamp_millis();
    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_upd_user_{}", now_tag),
            email: format!("itest_upd+{}@example.com", now_tag),
            full_name: Some("Before Update".to_string()),
            address: None,
            phone: None,
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user should succeed");
    let user_id = user_res.into_inner().items[0].user_id;

    let _ = core_operations::handlers::users::update_user(
        &txn,
        Request::new(UpdateUserRequest {
            user_id,
            username: None,
            password_plain: None,
            email: None,
            full_name: Some("After Update".to_string()),
            address: None,
            phone: None,
            role_id: None,
        }),
    )
    .await
    .expect("update_user should succeed");

    let search_res = core_operations::handlers::users::search_user(
        &txn,
        Request::new(SearchUserRequest { user_id }),
    )
    .await
    .expect("search_user should succeed");
    let found = search_res.into_inner().items;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].full_name.as_deref(), Some("After Update"));

    txn.rollback().await.ok();
}

/// U1 – create_user rejects weak passwords according to validate_password_strength.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_user_create_rejects_weak_password() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_weak_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let result = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_weak_{}", now_tag),
            email: format!("itest_weak+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "short".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await;

    let err = result.expect_err("create_user should reject weak password");
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(
        err.message().to_lowercase().contains("password"),
        "error should mention password, got: {}",
        err.message()
    );

    txn.rollback().await.ok();
}

/// U2 – create_user rejects duplicate email/username (unique constraint errors).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_user_create_rejects_duplicate_email_and_username() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_dup_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let username = format!("itest_dup_user_{}", now_tag);
    let email = format!("itest_dup+{}@example.com", now_tag);

    let _ = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: username.clone(),
            email: email.clone(),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("first create_user should succeed");

    let dup_email = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_dup_other_{}", now_tag),
            email: email.clone(),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await;
    let err = dup_email.expect_err("duplicate email should be rejected");
    assert!(
        err.code() == Code::AlreadyExists || err.code() == Code::InvalidArgument,
        "expected AlreadyExists or InvalidArgument for duplicate email, got {:?}",
        err.code()
    );

    let dup_username = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: username.clone(),
            email: format!("itest_dup2+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await;
    let err = dup_username.expect_err("duplicate username should be rejected");
    assert!(
        err.code() == Code::AlreadyExists || err.code() == Code::InvalidArgument,
        "expected AlreadyExists or InvalidArgument for duplicate username, got {:?}",
        err.code()
    );

    txn.rollback().await.ok();
}

/// U3 – delete_user removes user and cascades to related data (e.g. cart rows deleted).
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_user_delete_cascades_to_cart() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_del_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_del_{}", now_tag),
            email: format!("itest_del+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user should succeed");
    let user_id = user_res.into_inner().items[0].user_id;

    let category = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_del_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert ProductCategories");

    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        sku: ActiveValue::Set(None),
        name: ActiveValue::Set("Delete User Product".to_string()),
        slug: ActiveValue::Set(None),
        description: ActiveValue::Set(None),
        price: ActiveValue::Set(None),
        price_paise: ActiveValue::Set(1_000),
        category_id: ActiveValue::Set(category.category_id),
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
        product_id: ActiveValue::Set(product.product_id),
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
    .expect("create_cart_item should succeed");

    let cart_before = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&txn)
        .await
        .expect("query cart");
    assert_eq!(
        cart_before.len(),
        1,
        "cart should have one item before delete"
    );

    let _ = core_operations::handlers::users::delete_user(
        &txn,
        Request::new(DeleteUserRequest { user_id }),
    )
    .await
    .expect("delete_user should succeed");

    let user_after = core_db_entities::entity::users::Entity::find_by_id(user_id)
        .one(&txn)
        .await
        .expect("query user");
    assert!(user_after.is_none(), "user should be deleted");

    let cart_after = cart::Entity::find()
        .filter(cart::Column::UserId.eq(user_id))
        .all(&txn)
        .await
        .expect("query cart");
    assert!(
        cart_after.is_empty(),
        "cart rows should be cascaded when user is deleted"
    );

    txn.rollback().await.ok();
}

/// U4 – update_user password change makes old credentials invalid, new password valid.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_user_update_password_invalidates_old_credentials() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_pw_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: format!("itest_pw_{}", now_tag),
            email: format!("itest_pw+{}@example.com", now_tag),
            full_name: None,
            address: None,
            phone: None,
            password_plain: "OldPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user should succeed");
    let user_id = user_res.into_inner().items[0].user_id;

    let _ = core_operations::handlers::users::update_user(
        &txn,
        Request::new(UpdateUserRequest {
            user_id,
            username: None,
            password_plain: Some("NewPass456!".to_string()),
            email: None,
            full_name: None,
            address: None,
            phone: None,
            role_id: None,
        }),
    )
    .await
    .expect("update_user should succeed");

    let user = core_db_entities::entity::users::Entity::find_by_id(user_id)
        .one(&txn)
        .await
        .expect("query user")
        .expect("user should exist");

    let old_valid = core_operations::auth::verify_password("OldPass123!", &user.password_hash);
    let new_valid = core_operations::auth::verify_password("NewPass456!", &user.password_hash);

    assert!(
        !old_valid.unwrap_or(false),
        "old password should no longer verify"
    );
    assert!(new_valid.unwrap_or(false), "new password should verify");

    txn.rollback().await.ok();
}

/// U5 – get_user_pii_export returns full, correct user information snapshot.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_user_get_pii_export_returns_full_snapshot() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let role = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_role_pii_{}", now_tag)),
    }
    .insert(&txn)
    .await
    .expect("insert UserRoles");

    let username = format!("itest_pii_{}", now_tag);
    let email = format!("itest_pii+{}@example.com", now_tag);
    let full_name = "PII Export User".to_string();
    let address = "42 Export Street".to_string();
    let phone = "9998887777".to_string();

    let user_res = core_operations::handlers::users::create_user(
        &txn,
        Request::new(CreateUserRequest {
            username: username.clone(),
            email: email.clone(),
            full_name: Some(full_name.clone()),
            address: Some(address.clone()),
            phone: Some(phone.clone()),
            password_plain: "StrongPass123!".to_string(),
            role_id: Some(role.role_id),
        }),
    )
    .await
    .expect("create_user should succeed");
    let user_id = user_res.into_inner().items[0].user_id;

    let pii_res = core_operations::handlers::users::get_user_pii_export(
        &txn,
        Request::new(GetUserPiiExportRequest { user_id }),
    )
    .await
    .expect("get_user_pii_export should succeed");
    let pii = pii_res.into_inner();

    assert_eq!(pii.user_id, user_id);
    assert_eq!(pii.email, email);
    assert_eq!(pii.full_name.as_deref(), Some(full_name.as_str()));
    assert_eq!(pii.address.as_deref(), Some(address.as_str()));
    assert_eq!(pii.phone.as_deref(), Some(phone.as_str()));
    assert!(!pii.create_date.is_empty(), "create_date should be set");

    txn.rollback().await.ok();
}
