//! Unit tests for user_roles handlers.

use core_db_entities::entity::user_roles;
use proto::proto::core::{
    CreateUserRoleRequest, DeleteUserRoleRequest, SearchUserRoleRequest, UserRolesResponse,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_user_role_inserts_and_returns_created_model() {
    use core_operations::handlers::user_roles::create_user_role;

    let model = user_roles::Model {
        role_id: 1,
        role_name: "admin".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateUserRoleRequest {
        role_name: "admin".into(),
    });
    let result = create_user_role(&txn, req).await;
    assert!(result.is_ok());
    let UserRolesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].role_id, 1);
}

#[tokio::test]
async fn delete_user_role_not_found_yields_not_found_status() {
    use core_operations::handlers::user_roles::delete_user_role;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<user_roles::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteUserRoleRequest { role_id: 99 });
    let result = delete_user_role(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_user_role_filters_by_id_when_nonzero() {
    use core_operations::handlers::user_roles::search_user_role;

    let model = user_roles::Model {
        role_id: 3,
        role_name: "viewer".into(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchUserRoleRequest { role_id: 3 });
    let result = search_user_role(&txn, req).await;
    assert!(result.is_ok());
    let UserRolesResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].role_id, 3);
}

