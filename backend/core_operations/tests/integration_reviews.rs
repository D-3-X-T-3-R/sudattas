//! Integration tests for reviews: create, search, update, delete, admin_update_review_status.
//!
//! **Setup**
//! - Set `TEST_DATABASE_URL` or `DATABASE_URL`.
//! - Schema must be loaded (e.g. migrations or `backend/database/sql_dump/01_schema.sql`).
//!
//! **Run**
//! - `cargo test --test integration_reviews -- --ignored`

mod integration_common;

use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::FulfillmentStatus;
use core_db_entities::entity::{
    order_details, order_status, orders, product_categories, product_variants, shipping_addresses,
};
use integration_common::test_db_url;
use proto::proto::core::{
    AdminUpdateReviewStatusRequest, CreateProductRequest, CreateReviewRequest, CreateUserRequest,
    DeleteReviewRequest, ProductRatingSummaryRequest, SearchReviewRequest, UpdateReviewRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, Database, TransactionTrait};
use tonic::Request;

/// Create user + category + product; return (user_id, product_id). Does NOT create any order —
/// use this directly (not `reviews_test_setup`) for tests that need a customer who has *not*
/// purchased the product.
async fn create_user_and_product(txn: &sea_orm::DatabaseTransaction, now_tag: i64) -> (i64, i64) {
    let role = core_db_entities::entity::user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_rv_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_rv_{}", now_tag),
            email: format!("itest_rv+{}@example.com", now_tag),
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

    let cat = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(format!("itest_cat_rv_{}", now_tag)),
    }
    .insert(txn)
    .await
    .expect("insert ProductCategories");

    let create_res = core_operations::handlers::products::create_product(
        txn,
        Request::new(CreateProductRequest {
            name: format!("Review Product {}", now_tag),
            description: None,
            price_paise: 2_000,
            category_id: cat.category_id,
            sku: None,
            slug: None,
            fabric: None,
            weave: None,
            occasion: None,
            has_blouse_piece: None,
            care_instructions: None,
            product_status_id: None,
        }),
    )
    .await
    .expect("create_product");
    let product_id = create_res.into_inner().items[0].product_id;

    (user_id, product_id)
}

/// Seed a delivered order for `user_id` containing one active line item for `product_id` — the
/// minimum `create_review` now requires (see `create_review.rs`'s `has_delivered_purchase`).
/// `tag` must be unique per call (not just per test) since it feeds Orders' UNIQUE
/// `public_order_ref` — pass something like `"{now_tag}_{suffix}"` when seeding more than one
/// reviewer under the same `now_tag`.
async fn seed_delivered_purchase(
    txn: &sea_orm::DatabaseTransaction,
    user_id: i64,
    product_id: i64,
    tag: &str,
) {
    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert ProductVariants");

    let address = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        is_default: ActiveValue::Set(1),
        country: ActiveValue::Set("India".to_string()),
        state_region: ActiveValue::Set("Karnataka".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(Some(format!("Road {tag}"))),
        apartment_no_or_name: ActiveValue::Set(None),
        recipient_name: ActiveValue::Set(Some("Review Test".to_string())),
        phone_number: ActiveValue::Set(Some("+919999999999".to_string())),
    }
    .insert(txn)
    .await
    .expect("insert ShippingAddresses");

    let status = order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(format!("itest_rv_status_{tag}")),
    }
    .insert(txn)
    .await
    .expect("insert OrderStatus");

    let now = Utc::now();
    let order = orders::ActiveModel {
        order_id: ActiveValue::NotSet,
        order_number: ActiveValue::Set(None),
        public_order_ref: ActiveValue::Set(format!("ITEST_RV_REF_{tag}")),
        user_id: ActiveValue::Set(user_id),
        order_date: ActiveValue::Set(now),
        created_at: ActiveValue::Set(now),
        cancel_window_ends_at: ActiveValue::Set(None),
        earliest_booking_at: ActiveValue::Set(None),
        pickup_target_at: ActiveValue::Set(None),
        pickup_target_reason: ActiveValue::Set(None),
        pickup_target_set_by: ActiveValue::Set(None),
        pickup_target_updated_at: ActiveValue::Set(None),
        shipping_address_id: ActiveValue::Set(address.shipping_address_id),
        total_amount: ActiveValue::Set(None),
        status_id: ActiveValue::Set(status.status_id),
        payment_status: ActiveValue::Set(None),
        payment_method: ActiveValue::Set(Some("prepaid".to_string())),
        currency: ActiveValue::Set(Some("INR".to_string())),
        updated_at: ActiveValue::Set(Some(now)),
        subtotal_minor: ActiveValue::Set(2_000),
        items_total_minor_before_discount: ActiveValue::Set(Some(2_000)),
        shipping_minor: ActiveValue::Set(Some(0)),
        shipping_charge_minor: ActiveValue::Set(Some(0)),
        tax_total_minor: ActiveValue::Set(Some(0)),
        discount_total_minor: ActiveValue::Set(Some(0)),
        items_total_minor_after_discount: ActiveValue::Set(Some(2_000)),
        grand_total_minor: ActiveValue::Set(2_000),
        invoice_id: ActiveValue::Set(None),
        invoice_number: ActiveValue::Set(None),
        invoice_generated_at: ActiveValue::Set(None),
        invoice_storage_path: ActiveValue::Set(None),
        applied_coupon_id: ActiveValue::Set(None),
        applied_coupon_code: ActiveValue::Set(None),
        applied_discount_paise: ActiveValue::Set(None),
        refund_settlement_status: ActiveValue::Set(None),
        // The gate `create_review` checks — see has_delivered_purchase.
        fulfillment_status: ActiveValue::Set(FulfillmentStatus::Delivered),
    }
    .insert(txn)
    .await
    .expect("insert Orders");

    order_details::ActiveModel {
        order_detail_id: ActiveValue::NotSet,
        order_id: ActiveValue::Set(order.order_id),
        variant_id: ActiveValue::Set(variant.variant_id),
        quantity: ActiveValue::Set(1),
        price: ActiveValue::Set(None),
        line_total_minor: ActiveValue::Set(2_000),
        unit_price_minor: ActiveValue::Set(2_000),
        discount_minor: ActiveValue::Set(Some(0)),
        tax_minor: ActiveValue::Set(Some(0)),
        sku: ActiveValue::Set(None),
        title: ActiveValue::Set(None),
        line_attrs: ActiveValue::Set(None),
        item_status: ActiveValue::Set("active".to_string()),
        cancelled_at: ActiveValue::Set(None),
    }
    .insert(txn)
    .await
    .expect("insert OrderDetails");
}

/// Create user + category + product + a delivered order for that product — the setup every
/// existing review test needs now that `create_review` requires a delivered purchase.
/// Returns (user_id, product_id).
async fn reviews_test_setup(txn: &sea_orm::DatabaseTransaction, now_tag: i64) -> (i64, i64) {
    let (user_id, product_id) = create_user_and_product(txn, now_tag).await;
    seed_delivered_purchase(txn, user_id, product_id, &now_tag.to_string()).await;
    (user_id, product_id)
}

/// Create one more user (distinct from `reviews_test_setup`'s) with their own delivered purchase
/// of `product_id`, so multiple ratings can be left on the same product — `Reviews` has
/// UNIQUE(UserID, ProductID), so a second rating from the same user must go through
/// update_review, not another create_review, and each reviewer needs their own delivered order
/// now that create_review requires one.
async fn create_extra_reviewer(
    txn: &sea_orm::DatabaseTransaction,
    now_tag: i64,
    suffix: &str,
    product_id: i64,
) -> i64 {
    let role = core_db_entities::entity::user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(format!("itest_rv_{}_{}", now_tag, suffix)),
    }
    .insert(txn)
    .await
    .expect("insert UserRoles");

    let user_res = core_operations::handlers::users::create_user(
        txn,
        Request::new(CreateUserRequest {
            username: format!("itest_rv_{}_{}", now_tag, suffix),
            email: format!("itest_rv+{}_{}@example.com", now_tag, suffix),
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

    seed_delivered_purchase(txn, user_id, product_id, &format!("{}_{}", now_tag, suffix)).await;
    user_id
}

/// RV1 – create_review + search_review by product_id returns the review.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_review_search_by_product_id_returns_review() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let create_res = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 4,
            comment: "Great product.".to_string(),
        }),
    )
    .await
    .expect("create_review should succeed");
    let created = create_res.into_inner().items[0].clone();
    assert_eq!(created.product_id, product_id);
    assert_eq!(created.user_id, user_id);
    assert_eq!(created.rating, 4);
    assert_eq!(created.comment, "Great product.");

    let search_res = core_operations::handlers::reviews::search_review(
        &txn,
        Request::new(SearchReviewRequest {
            review_id: 0,
            product_id: Some(product_id),
            user_id: None,
            limit: Some(10),
            offset: None,
            status_filter: None,
        }),
    )
    .await
    .expect("search_review should succeed");
    let items = search_res.into_inner().items;
    assert!(!items.is_empty());
    let found = items
        .iter()
        .find(|r| r.review_id == created.review_id)
        .expect("review in results");
    assert_eq!(found.product_id, product_id);
    assert_eq!(found.rating, 4);
    assert_eq!(found.comment, "Great product.");

    txn.rollback().await.ok();
}

/// RV2 – update_review modifies rating/body; search_review reflects the updated content.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_update_review_search_reflects_updated_content() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let create_res = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 3,
            comment: "Original comment.".to_string(),
        }),
    )
    .await
    .expect("create_review should succeed");
    let review_id = create_res.into_inner().items[0].review_id;

    let _ = core_operations::handlers::reviews::update_review(
        &txn,
        Request::new(UpdateReviewRequest {
            review_id,
            product_id: None,
            user_id: None,
            rating: Some(5),
            comment: Some("Updated: excellent!".to_string()),
        }),
    )
    .await
    .expect("update_review should succeed");

    let search_res = core_operations::handlers::reviews::search_review(
        &txn,
        Request::new(SearchReviewRequest {
            review_id: 0,
            product_id: Some(product_id),
            user_id: None,
            limit: Some(10),
            offset: None,
            status_filter: None,
        }),
    )
    .await
    .expect("search_review should succeed");
    let items = search_res.into_inner().items;
    let found = items
        .iter()
        .find(|r| r.review_id == review_id)
        .expect("review in results");
    assert_eq!(found.rating, 5);
    assert_eq!(found.comment, "Updated: excellent!");

    txn.rollback().await.ok();
}

/// RV3 – delete_review removes review so search_review no longer returns it.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_delete_review_search_no_longer_returns_it() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let create_res = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 2,
            comment: "Will be deleted.".to_string(),
        }),
    )
    .await
    .expect("create_review should succeed");
    let review_id = create_res.into_inner().items[0].review_id;

    let _ = core_operations::handlers::reviews::delete_review(
        &txn,
        Request::new(DeleteReviewRequest { review_id }),
    )
    .await
    .expect("delete_review should succeed");

    let search_res = core_operations::handlers::reviews::search_review(
        &txn,
        Request::new(SearchReviewRequest {
            review_id: 0,
            product_id: Some(product_id),
            user_id: None,
            limit: Some(10),
            offset: None,
            status_filter: None,
        }),
    )
    .await
    .expect("search_review should succeed");
    let items = search_res.into_inner().items;
    assert!(
        !items.iter().any(|r| r.review_id == review_id),
        "search_review should not return deleted review"
    );

    txn.rollback().await.ok();
}

/// RV4 – admin_update_review_status flips status (e.g. pending → approved) and persists correctly.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_admin_update_review_status_persists() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let create_res = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 5,
            comment: "Moderation test.".to_string(),
        }),
    )
    .await
    .expect("create_review should succeed");
    let review_id = create_res.into_inner().items[0].review_id;

    let admin_res = core_operations::handlers::reviews::admin_update_review_status(
        &txn,
        Request::new(AdminUpdateReviewStatusRequest {
            review_id,
            status: "approved".to_string(),
        }),
    )
    .await
    .expect("admin_update_review_status should succeed");
    assert!(admin_res.into_inner().success);

    let search_res = core_operations::handlers::reviews::search_review(
        &txn,
        Request::new(SearchReviewRequest {
            review_id: 0,
            product_id: Some(product_id),
            user_id: None,
            limit: Some(10),
            offset: None,
            status_filter: Some("approved".to_string()),
        }),
    )
    .await
    .expect("search_review should succeed");
    let items = search_res.into_inner().items;
    let found = items.iter().find(|r| r.review_id == review_id);
    assert!(
        found.is_some(),
        "search with status_filter=approved should return the review after admin approval"
    );

    txn.rollback().await.ok();
}

/// RV5 – get_product_rating_summary ceil-rounds the average (3, 4, 5 -> avg 4.0 -> ceil 4) and
/// returns the real count, across ratings from multiple distinct customers on the same product.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_get_product_rating_summary_ceils_the_average() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;
    let user_id_2 = create_extra_reviewer(&txn, now_tag, "b", product_id).await;
    let user_id_3 = create_extra_reviewer(&txn, now_tag, "c", product_id).await;

    for (uid, rating) in [(user_id, 3), (user_id_2, 4), (user_id_3, 5)] {
        core_operations::handlers::reviews::create_review(
            &txn,
            Request::new(CreateReviewRequest {
                product_id,
                user_id: uid,
                rating,
                comment: String::new(),
            }),
        )
        .await
        .expect("create_review should succeed");
    }

    let summary = core_operations::handlers::reviews::get_product_rating_summary(
        &txn,
        Request::new(ProductRatingSummaryRequest { product_id }),
    )
    .await
    .expect("get_product_rating_summary should succeed")
    .into_inner();

    assert_eq!(summary.product_id, product_id);
    assert_eq!(summary.rating_count, 3);
    // avg(3, 4, 5) = 4.0 exactly, ceil(4.0) = 4.
    assert_eq!(summary.average_rating, 4);

    txn.rollback().await.ok();
}

/// RV6 – get_product_rating_summary on a product with no ratings returns 0/0, and a non-exact
/// average (avg(3, 4) = 3.5) rounds up to 4, not down to 3 — this is the case the feature exists
/// for: both 3.2-ish and 3.8-ish averages must ceil to 4.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_get_product_rating_summary_zero_and_fractional_average() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = reviews_test_setup(&txn, now_tag).await;

    let empty_summary = core_operations::handlers::reviews::get_product_rating_summary(
        &txn,
        Request::new(ProductRatingSummaryRequest { product_id }),
    )
    .await
    .expect("get_product_rating_summary should succeed")
    .into_inner();
    assert_eq!(empty_summary.rating_count, 0);
    assert_eq!(empty_summary.average_rating, 0);

    let user_id_2 = create_extra_reviewer(&txn, now_tag, "b", product_id).await;
    for (uid, rating) in [(user_id, 3), (user_id_2, 4)] {
        core_operations::handlers::reviews::create_review(
            &txn,
            Request::new(CreateReviewRequest {
                product_id,
                user_id: uid,
                rating,
                comment: String::new(),
            }),
        )
        .await
        .expect("create_review should succeed");
    }

    let summary = core_operations::handlers::reviews::get_product_rating_summary(
        &txn,
        Request::new(ProductRatingSummaryRequest { product_id }),
    )
    .await
    .expect("get_product_rating_summary should succeed")
    .into_inner();
    assert_eq!(summary.rating_count, 2);
    // avg(3, 4) = 3.5, ceil(3.5) = 4.
    assert_eq!(summary.average_rating, 4);

    txn.rollback().await.ok();
}

/// RV7 – create_review is rejected for a customer who has never ordered the product, and again
/// for one whose order for it hasn't reached `delivered` yet (still fulfillment_status='booked').
/// This is the actual security gate this feature exists for.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_create_review_rejected_without_a_delivered_purchase() {
    let db = Database::connect(&test_db_url())
        .await
        .expect("connect to test DB");
    let txn = db.begin().await.expect("begin transaction");

    let now_tag = Utc::now().timestamp_millis();
    let (user_id, product_id) = create_user_and_product(&txn, now_tag).await;

    // No order at all yet.
    let err = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 5,
            comment: String::new(),
        }),
    )
    .await
    .expect_err("create_review must reject a customer who never ordered this product");
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);

    // An order exists but hasn't been delivered yet (still in transit) — must still be rejected.
    let variant = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(product_id),
        size_id: ActiveValue::Set(None),
        color_id: ActiveValue::Set(None),
        additional_price: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert ProductVariants");
    let address = shipping_addresses::ActiveModel {
        shipping_address_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(Some(user_id)),
        is_default: ActiveValue::Set(1),
        country: ActiveValue::Set("India".to_string()),
        state_region: ActiveValue::Set("Karnataka".to_string()),
        city: ActiveValue::Set("Bengaluru".to_string()),
        postal_code: ActiveValue::Set("560001".to_string()),
        road: ActiveValue::Set(None),
        apartment_no_or_name: ActiveValue::Set(None),
        recipient_name: ActiveValue::Set(None),
        phone_number: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert ShippingAddresses");
    let status = order_status::ActiveModel {
        status_id: ActiveValue::NotSet,
        status_name: ActiveValue::Set(format!("itest_rv_status_undelivered_{now_tag}")),
    }
    .insert(&txn)
    .await
    .expect("insert OrderStatus");
    let now = Utc::now();
    let order = orders::ActiveModel {
        order_id: ActiveValue::NotSet,
        order_number: ActiveValue::Set(None),
        public_order_ref: ActiveValue::Set(format!("ITEST_RV_UNDELIVERED_{now_tag}")),
        user_id: ActiveValue::Set(user_id),
        order_date: ActiveValue::Set(now),
        created_at: ActiveValue::Set(now),
        cancel_window_ends_at: ActiveValue::Set(None),
        earliest_booking_at: ActiveValue::Set(None),
        pickup_target_at: ActiveValue::Set(None),
        pickup_target_reason: ActiveValue::Set(None),
        pickup_target_set_by: ActiveValue::Set(None),
        pickup_target_updated_at: ActiveValue::Set(None),
        shipping_address_id: ActiveValue::Set(address.shipping_address_id),
        total_amount: ActiveValue::Set(None),
        status_id: ActiveValue::Set(status.status_id),
        payment_status: ActiveValue::Set(None),
        payment_method: ActiveValue::Set(Some("prepaid".to_string())),
        currency: ActiveValue::Set(Some("INR".to_string())),
        updated_at: ActiveValue::Set(Some(now)),
        subtotal_minor: ActiveValue::Set(2_000),
        items_total_minor_before_discount: ActiveValue::Set(Some(2_000)),
        shipping_minor: ActiveValue::Set(Some(0)),
        shipping_charge_minor: ActiveValue::Set(Some(0)),
        tax_total_minor: ActiveValue::Set(Some(0)),
        discount_total_minor: ActiveValue::Set(Some(0)),
        items_total_minor_after_discount: ActiveValue::Set(Some(2_000)),
        grand_total_minor: ActiveValue::Set(2_000),
        invoice_id: ActiveValue::Set(None),
        invoice_number: ActiveValue::Set(None),
        invoice_generated_at: ActiveValue::Set(None),
        invoice_storage_path: ActiveValue::Set(None),
        applied_coupon_id: ActiveValue::Set(None),
        applied_coupon_code: ActiveValue::Set(None),
        applied_discount_paise: ActiveValue::Set(None),
        refund_settlement_status: ActiveValue::Set(None),
        // Not delivered yet — must still fail the gate.
        fulfillment_status: ActiveValue::Set(FulfillmentStatus::InTransit),
    }
    .insert(&txn)
    .await
    .expect("insert Orders");
    order_details::ActiveModel {
        order_detail_id: ActiveValue::NotSet,
        order_id: ActiveValue::Set(order.order_id),
        variant_id: ActiveValue::Set(variant.variant_id),
        quantity: ActiveValue::Set(1),
        price: ActiveValue::Set(None),
        line_total_minor: ActiveValue::Set(2_000),
        unit_price_minor: ActiveValue::Set(2_000),
        discount_minor: ActiveValue::Set(Some(0)),
        tax_minor: ActiveValue::Set(Some(0)),
        sku: ActiveValue::Set(None),
        title: ActiveValue::Set(None),
        line_attrs: ActiveValue::Set(None),
        item_status: ActiveValue::Set("active".to_string()),
        cancelled_at: ActiveValue::Set(None),
    }
    .insert(&txn)
    .await
    .expect("insert OrderDetails");

    let err = core_operations::handlers::reviews::create_review(
        &txn,
        Request::new(CreateReviewRequest {
            product_id,
            user_id,
            rating: 5,
            comment: String::new(),
        }),
    )
    .await
    .expect_err("create_review must reject an order that hasn't reached delivered yet");
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);

    txn.rollback().await.ok();
}
