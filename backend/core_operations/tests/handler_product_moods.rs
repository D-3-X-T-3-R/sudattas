//! Unit tests for product_moods handlers.

use core_db_entities::entity::product_moods;
use proto::proto::core::{
    CreateProductMoodRequest, DeleteProductMoodRequest, ProductMoodsResponse,
    SearchProductMoodRequest, UpdateProductMoodRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_product_mood_inserts_and_returns_created_model() {
    use core_operations::handlers::product_moods::create_product_mood;

    let model = product_moods::Model {
        mood_id: 1,
        mood_name: "Elegant".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateProductMoodRequest {
        mood_name: "Elegant".into(),
    });
    let result = create_product_mood(&txn, req).await;
    assert!(result.is_ok());
    let ProductMoodsResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].mood_name, "Elegant");
}

#[tokio::test]
async fn update_product_mood_not_found_yields_not_found_status() {
    use core_operations::handlers::product_moods::update_product_mood;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_moods::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateProductMoodRequest {
        mood_id: 99,
        mood_name: Some("Relaxed".into()),
    });
    let result = update_product_mood(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_product_mood_not_found_yields_not_found_status() {
    use core_operations::handlers::product_moods::delete_product_mood;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<product_moods::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteProductMoodRequest { mood_id: 77 });
    let result = delete_product_mood(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_product_mood_filters_by_id_and_name() {
    use core_operations::handlers::product_moods::search_product_mood;

    let model = product_moods::Model {
        mood_id: 3,
        mood_name: "Festive".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchProductMoodRequest {
        mood_id: Some(3),
        mood_name: Some("Festive".into()),
    });
    let result = search_product_mood(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].mood_id, 3);
}
