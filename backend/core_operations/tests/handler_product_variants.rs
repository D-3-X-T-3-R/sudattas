//! Unit tests for product_variants handlers using SeaORM MockDatabase.

use core_db_entities::entity::{order_details, product_variants};
use proto::proto::core::{
    CreateProductVariantRequest, DeleteProductVariantRequest, ProductVariantsResponse,
    UpdateProductVariantRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn delete_product_variant_success_returns_deleted_item() {
    use core_operations::handlers::product_variants::delete_product_variant;

    let model = product_variants::Model {
        variant_id: 5,
        product_id: 100,
        size_id: Some(2),
        color_id: Some(3),
        additional_price: Some(250),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        // 1st query: find_by_id returns the model.
        .append_query_results(vec![vec![model]])
        // 2nd query: order_details lookup for the order-history guard — none found.
        .append_query_results(vec![Vec::<order_details::Model>::new()])
        // Remaining ops: cart/inventory_log/inventory cascade deletes, then the variant delete.
        .append_exec_results(vec![
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            },
            MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            },
        ])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(DeleteProductVariantRequest { variant_id: 5 });
    let result = delete_product_variant(&txn, req).await;
    assert!(
        result.is_ok(),
        "delete_product_variant should succeed for existing variant"
    );
    let resp = result.unwrap().into_inner();
    assert_eq!(resp.items.len(), 1);
    let item = &resp.items[0];
    assert_eq!(item.variant_id, 5);
    assert_eq!(item.product_id, 100);
    assert_eq!(item.size_id, Some(2));
    assert_eq!(item.color_id, Some(3));
    assert_eq!(item.additional_price_paise, Some(250_i64));
}

#[tokio::test]
async fn delete_product_variant_not_found_returns_not_found_status() {
    use core_operations::handlers::product_variants::delete_product_variant;

    // Query returns no rows for the given variant id.
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_variants::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(DeleteProductVariantRequest { variant_id: 999 });
    let result = delete_product_variant(&txn, req).await;
    assert!(result.is_err(), "expected not_found for missing variant");
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_product_variant_with_order_history_is_refused() {
    use core_db_entities::entity::order_details;
    use core_operations::handlers::product_variants::delete_product_variant;

    let model = product_variants::Model {
        variant_id: 6,
        product_id: 100,
        size_id: Some(2),
        color_id: Some(3),
        additional_price: Some(250),
    };
    let order_detail = order_details::Model {
        order_detail_id: 1,
        order_id: 1,
        variant_id: 6,
        quantity: 1,
        price: None,
        line_total_minor: 1000,
        unit_price_minor: 1000,
        discount_minor: None,
        tax_minor: None,
        sku: None,
        title: None,
        line_attrs: None,
        item_status: "active".to_string(),
        cancelled_at: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .append_query_results(vec![vec![order_detail]])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(DeleteProductVariantRequest { variant_id: 6 });
    let result = delete_product_variant(&txn, req).await;
    assert!(
        result.is_err(),
        "expected refusal for a variant with order history"
    );
    assert_eq!(result.unwrap_err().code(), tonic::Code::FailedPrecondition);
}

#[tokio::test]
async fn create_product_variant_inserts_and_returns_created_model() {
    use core_operations::handlers::product_variants::create_product_variant;

    let model = product_variants::Model {
        variant_id: 7,
        product_id: 100,
        size_id: Some(2),
        color_id: Some(3),
        additional_price: Some(150),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        // 1st query: duplicate-variant guard — none found.
        .append_query_results(vec![Vec::<product_variants::Model>::new()])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 7,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CreateProductVariantRequest {
        product_id: 100,
        size_id: Some(2),
        color_id: Some(3),
        additional_price_paise: Some(150),
    });

    let result = create_product_variant(&txn, req).await;
    assert!(result.is_ok());
    let ProductVariantsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let v = &items[0];
    assert_eq!(v.variant_id, 7);
    assert_eq!(v.product_id, 100);
    assert_eq!(v.size_id, Some(2));
    assert_eq!(v.color_id, Some(3));
    assert_eq!(v.additional_price_paise, Some(150));
}

#[tokio::test]
async fn create_product_variant_rejects_duplicate_size_color_combo() {
    use core_operations::handlers::product_variants::create_product_variant;

    let existing = product_variants::Model {
        variant_id: 20,
        product_id: 100,
        size_id: Some(1),
        color_id: None,
        additional_price: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        // Duplicate-variant guard finds the existing "Free Size" variant on this product.
        .append_query_results(vec![vec![existing]])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(CreateProductVariantRequest {
        product_id: 100,
        size_id: Some(1),
        color_id: None,
        additional_price_paise: None,
    });

    let result = create_product_variant(&txn, req).await;
    assert!(
        result.is_err(),
        "expected duplicate size/color to be rejected"
    );
    assert_eq!(result.unwrap_err().code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn update_product_variant_rejects_duplicate_size_color_combo() {
    use core_operations::handlers::product_variants::update_product_variant;

    let existing = product_variants::Model {
        variant_id: 21,
        product_id: 100,
        size_id: Some(2),
        color_id: None,
        additional_price: None,
    };
    let other_variant_with_target_size = product_variants::Model {
        variant_id: 22,
        product_id: 100,
        size_id: Some(1),
        color_id: None,
        additional_price: None,
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        // find_by_id for variant 21 (the one being updated).
        .append_query_results(vec![vec![existing]])
        // Duplicate-variant guard finds variant 22, already using the target size/color.
        .append_query_results(vec![vec![other_variant_with_target_size]])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(UpdateProductVariantRequest {
        variant_id: 21,
        product_id: None,
        size_id: Some(1),
        color_id: None,
        additional_price_paise: None,
    });

    let result = update_product_variant(&txn, req).await;
    assert!(
        result.is_err(),
        "expected duplicate size/color to be rejected"
    );
    assert_eq!(result.unwrap_err().code(), tonic::Code::AlreadyExists);
}

#[tokio::test]
async fn update_product_variant_not_found_yields_not_found_status() {
    use core_operations::handlers::product_variants::update_product_variant;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_variants::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(UpdateProductVariantRequest {
        variant_id: 42,
        product_id: None,
        size_id: None,
        color_id: None,
        additional_price_paise: None,
    });
    let result = update_product_variant(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_product_variant_updates_fields_and_preserves_existing_when_missing() {
    use core_operations::handlers::product_variants::update_product_variant;

    let existing = product_variants::Model {
        variant_id: 8,
        product_id: 200,
        size_id: Some(3),
        color_id: Some(4),
        additional_price: Some(100),
    };
    let updated = product_variants::Model {
        variant_id: 8,
        product_id: 201,
        size_id: Some(3),
        color_id: Some(5),
        additional_price: Some(250),
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        // 1st query: find_by_id (the existing variant).
        .append_query_results(vec![vec![existing]])
        // 2nd query: duplicate-variant guard — none found.
        .append_query_results(vec![Vec::<product_variants::Model>::new()])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin transaction");

    let req = Request::new(UpdateProductVariantRequest {
        variant_id: 8,
        product_id: Some(201),
        size_id: None,
        color_id: Some(5),
        additional_price_paise: Some(250),
    });

    let result = update_product_variant(&txn, req).await;
    assert!(result.is_ok());
    let ProductVariantsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let v = &items[0];
    assert_eq!(v.variant_id, 8);
    assert_eq!(v.product_id, 201);
    assert_eq!(v.size_id, Some(3));
    assert_eq!(v.color_id, Some(5));
    assert_eq!(v.additional_price_paise, Some(250));
}
