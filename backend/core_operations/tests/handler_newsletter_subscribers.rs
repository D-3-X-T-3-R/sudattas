//! Unit tests for newsletter_subscribers handlers using SeaORM MockDatabase.

use core_db_entities::entity::newsletter_subscribers;
use proto::proto::core::{
    CreateNewsletterSubscriberRequest, DeleteNewsletterSubscriberRequest,
    NewsletterSubscribersResponse, SearchNewsletterSubscriberRequest,
    UpdateNewsletterSubscriberRequest,
};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, TransactionTrait};
use tonic::Request;

#[tokio::test]
async fn create_newsletter_subscriber_inserts_and_returns_created_model() {
    use core_operations::handlers::newsletter_subscribers::create_newsletter_subscriber;

    let model = newsletter_subscribers::Model {
        subscriber_id: 1,
        email: "user@example.com".into(),
        subscription_date: chrono::Utc::now(),
        unsubscribed_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(CreateNewsletterSubscriberRequest {
        email: "user@example.com".into(),
    });
    let result = create_newsletter_subscriber(&txn, req).await;
    assert!(result.is_ok());
    let NewsletterSubscribersResponse { items } = result.unwrap().into_inner();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].subscriber_id, 1);
}

#[tokio::test]
async fn update_newsletter_subscriber_not_found_yields_not_found_status() {
    use core_operations::handlers::newsletter_subscribers::update_newsletter_subscriber;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<newsletter_subscribers::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(UpdateNewsletterSubscriberRequest {
        subscriber_id: 99,
        email: "new@example.com".into(),
    });
    let result = update_newsletter_subscriber(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn delete_newsletter_subscriber_not_found_yields_not_found_status() {
    use core_operations::handlers::newsletter_subscribers::delete_newsletter_subscriber;

    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![Vec::<newsletter_subscribers::Model>::new()])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(DeleteNewsletterSubscriberRequest { subscriber_id: 77 });
    let result = delete_newsletter_subscriber(&txn, req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn search_newsletter_subscriber_filters_by_id_when_nonzero() {
    use core_operations::handlers::newsletter_subscribers::search_newsletter_subscriber;

    let model = newsletter_subscribers::Model {
        subscriber_id: 5,
        email: "findme@example.com".into(),
        subscription_date: chrono::Utc::now(),
        unsubscribed_at: None,
    };
    let db = MockDatabase::new(DatabaseBackend::MySql)
        .append_query_results(vec![vec![model]])
        .into_connection();
    let txn = db.begin().await.expect("begin");

    let req = Request::new(SearchNewsletterSubscriberRequest { subscriber_id: 5 });
    let result = search_newsletter_subscriber(&txn, req).await;
    assert!(result.is_ok());
    let res = result.unwrap().into_inner();
    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].subscriber_id, 5);
}
