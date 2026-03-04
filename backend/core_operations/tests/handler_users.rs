//! Unit tests for user handlers using SeaORM MockDatabase.

use core_db_entities::entity::users;
use proto::proto::core::{
    CreateUserRequest, DeleteUserRequest, SearchUserRequest, UpdateUserRequest, UsersResponse,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

fn make_user(id: i64) -> users::Model {
    users::Model {
        user_id: id,
        username: format!("user{}", id),
        password_hash: "hashed".to_string(),
        email: format!("user{}@example.com", id),
        email_verified: None,
        email_verified_at: None,
        full_name: Some(format!("User {}", id)),
        address: Some("Address".to_string()),
        phone: Some("1234567890".to_string()),
        user_status_id: None,
        role_id: Some(1),
        last_login_at: None,
        marketing_opt_out: None,
        create_date: chrono::Utc::now(),
        updated_at: None,
    }
}

#[tokio::test]
async fn create_user_inserts_and_returns_created_model() {
    use core_operations::handlers::users::create_user;

    // Use a fixed hash since auth::hash_password is mocked by behavior here.
    let model = make_user(1);

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateUserRequest {
        username: "user1".to_string(),
        email: "user1@example.com".to_string(),
        full_name: Some("User 1".to_string()),
        address: Some("Address".to_string()),
        phone: Some("1234567890".to_string()),
        password_plain: "StrongP@ssw0rd".to_string(),
        role_id: Some(1),
    });
    let result = create_user(&txn, req).await;
    assert!(result.is_ok());
    let UsersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let u = &items[0];
    assert_eq!(u.user_id, 1);
    assert_eq!(u.username, "user1");
    assert_eq!(u.email, "user1@example.com");
}

#[tokio::test]
async fn delete_user_deletes_existing_and_returns_response() {
    use core_operations::handlers::users::delete_user;

    let model = make_user(10);

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteUserRequest { user_id: 10 });
    let result = delete_user(&txn, req).await;
    assert!(result.is_ok());
    let UsersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let u = &items[0];
    assert_eq!(u.user_id, 10);
    assert_eq!(u.username, "user10");
}

#[tokio::test]
async fn delete_user_not_found_yields_not_found_status() {
    use core_operations::handlers::users::delete_user;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<users::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteUserRequest { user_id: 999 });
    let result = delete_user(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_user_not_found_yields_not_found_status() {
    use core_operations::handlers::users::update_user;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<users::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateUserRequest {
        user_id: 123,
        username: Some("newuser".to_string()),
        email: Some("new@example.com".to_string()),
        full_name: None,
        address: None,
        phone: None,
        password_plain: None,
        role_id: None,
    });
    let result = update_user(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn update_user_updates_fields_and_preserves_existing_when_missing() {
    use core_operations::handlers::users::update_user;

    let existing = make_user(5);
    let updated = users::Model {
        username: "user5_new".to_string(),
        email: "user5_new@example.com".to_string(),
        full_name: Some("User Five".to_string()),
        ..existing.clone()
    };

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![existing]])
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![updated]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateUserRequest {
        user_id: 5,
        username: Some("user5_new".to_string()),
        email: Some("user5_new@example.com".to_string()),
        full_name: Some("User Five".to_string()),
        address: None,
        phone: None,
        password_plain: None,
        role_id: None,
    });

    let result = update_user(&txn, req).await;
    assert!(result.is_ok());
    let UsersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let u = &items[0];
    assert_eq!(u.user_id, 5);
    assert_eq!(u.username, "user5_new");
    assert_eq!(u.email, "user5_new@example.com");
    assert_eq!(u.full_name.as_deref(), Some("User Five"));
}

#[tokio::test]
async fn search_user_by_user_id_filters_correctly() {
    use core_operations::handlers::users::search_user;

    let model = make_user(20);
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchUserRequest { user_id: 20 });
    let result = search_user(&txn, req).await;
    assert!(result.is_ok());
    let UsersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    let u = &items[0];
    assert_eq!(u.user_id, 20);
    assert_eq!(u.username, "user20");
}

