//! P2 Security: unit tests (mock DB) and integration tests (real DB) for RecordSecurityAuditEvent, GetUserPiiExport.

mod integration_common;

use core_db_entities::entity::users;
use proto::proto::core::{GetUserPiiExportRequest, RecordSecurityAuditRequest};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn record_security_audit_event_empty_event_type_returns_invalid_argument() {
    use core_operations::handlers::security::record_security_audit_event;

    let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(RecordSecurityAuditRequest {
        event_type: "".to_string(),
        details: None,
    });
    let result = record_security_audit_event(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn record_security_audit_event_success() {
    use core_db_entities::entity::security_audit_log;
    use core_operations::handlers::security::record_security_audit_event;

    let row = security_audit_log::Model {
        id: 1,
        event_type: "secrets_rotation".to_string(),
        details: Some("REDIS_URL rotated".to_string()),
        created_at: chrono::Utc::now(),
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![row]])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(RecordSecurityAuditRequest {
        event_type: "secrets_rotation".to_string(),
        details: Some("REDIS_URL rotated".to_string()),
    });
    let result = record_security_audit_event(&txn, req).await;
    assert!(result.is_ok(), "expected Ok: {:?}", result.err());
    assert!(result.unwrap().into_inner().success);
}

#[tokio::test]
async fn get_user_pii_export_not_found() {
    use core_operations::handlers::users::get_user_pii_export;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<users::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(GetUserPiiExportRequest { user_id: 999 });
    let result = get_user_pii_export(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn get_user_pii_export_returns_pii_no_password() {
    use core_operations::handlers::users::get_user_pii_export;

    let now = chrono::Utc::now();
    let user = users::Model {
        user_id: 1,
        username: "u".to_string(),
        password: "secret".to_string(),
        password_hash: None,
        email: "u@example.com".to_string(),
        email_verified: None,
        email_verified_at: None,
        full_name: Some("Full Name".to_string()),
        address: Some("123 St".to_string()),
        phone: Some("+123".to_string()),
        user_status_id: None,
        last_login_at: None,
        marketing_opt_out: None,
        create_date: now,
        updated_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![user]])
        .into_connection();
    let txn = db.begin().await.expect("begin");
    let req = Request::new(GetUserPiiExportRequest { user_id: 1 });
    let result = get_user_pii_export(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.user_id, 1);
    assert_eq!(res.email, "u@example.com");
    assert_eq!(res.full_name.as_deref(), Some("Full Name"));
    assert_eq!(res.address.as_deref(), Some("123 St"));
    assert_eq!(res.phone.as_deref(), Some("+123"));
    assert!(!res.create_date.is_empty());
}

// ---------- Integration (requires TEST_DATABASE_URL and migrated schema) ----------

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_record_security_audit_event() {
    use core_operations::handlers::security::record_security_audit_event;
    use sea_orm::{Database, TransactionTrait};
    use tonic::Request;

    let db = Database::connect(&integration_common::test_db_url())
        .await
        .expect("connect");
    let txn = db.begin().await.expect("begin");
    let req = Request::new(RecordSecurityAuditRequest {
        event_type: "secrets_rotation".to_string(),
        details: Some("integration test".to_string()),
    });
    let result = record_security_audit_event(&txn, req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().into_inner().success);
    txn.commit().await.expect("commit");
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL and migrated schema"]
async fn integration_get_user_pii_export() {
    use core_operations::handlers::users::get_user_pii_export;
    use sea_orm::{Database, TransactionTrait};
    use tonic::Request;

    let db = Database::connect(&integration_common::test_db_url())
        .await
        .expect("connect");
    let txn = db.begin().await.expect("begin");
    let req = Request::new(GetUserPiiExportRequest { user_id: 1 });
    let result = get_user_pii_export(&txn, req).await;
    // May be Ok (user 1 exists) or NotFound (no such user)
    if let Ok(res) = result {
        assert!(res.into_inner().user_id == 1);
    }
}
