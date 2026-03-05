//! Unit tests for user_activities handlers.

use core_db_entities::entity::user_activity;
use proto::proto::core::{
    CreateUserActivityRequest, DeleteUserActivityRequest, SearchUserActivityRequest,
    UpdateUserActivityRequest, UserActivitiesResponse,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

fn make_activity(id: i64) -> user_activity::Model {
    user_activity::Model {
        activity_id: id,
        user_id: Some(1),
        activity_type: "login".into(),
        activity_time: chrono::Utc::now(),
        activity_details: Some("details".into()),
    }
}

#[tokio::test]
async fn create_user_activity_inserts_and_returns_created_model() {
    use core_operations::handlers::user_activities::create_user_activity;

    let model = make_activity(1);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateUserActivityRequest {
        user_id: 1,
        activity_type: "login".into(),
        activity_details: "details".into(),
    });
    let result = create_user_activity(&txn, req).await;
    assert!(result.is_ok());
    let UserActivitiesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].activity_id, 1);
}

#[tokio::test]
async fn update_user_activity_not_found_yields_not_found_status() {
    use core_operations::handlers::user_activities::update_user_activity;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<user_activity::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateUserActivityRequest {
        activity_id: 99,
        user_id: None,
        activity_type: None,
        activity_details: None,
    });
    let result = update_user_activity(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_user_activity_not_found_yields_not_found_status() {
    use core_operations::handlers::user_activities::delete_user_activity;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<user_activity::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteUserActivityRequest { activity_id: 77 });
    let result = delete_user_activity(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_user_activity_filters_by_id_when_nonzero() {
    use core_operations::handlers::user_activities::search_user_activity;

    let model = make_activity(3);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchUserActivityRequest { activity_id: 3 });
    let result = search_user_activity(&txn, req).await;
    assert!(result.is_ok());
    let UserActivitiesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].activity_id, 3);
}
