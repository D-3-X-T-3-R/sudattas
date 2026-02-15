use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::newsletter_subscribers;
use proto::proto::core::{
    CreateNewsletterSubscriberRequest, NewsletterSubscriberResponse,
    NewsletterSubscribersResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_newsletter_subscriber(
    txn: &DatabaseTransaction,
    request: Request<CreateNewsletterSubscriberRequest>,
) -> Result<Response<NewsletterSubscribersResponse>, Status> {
    let req = request.into_inner();
    let model = newsletter_subscribers::ActiveModel {
        subscriber_id: ActiveValue::NotSet,
        email: ActiveValue::Set(req.email),
        subscription_date: ActiveValue::Set(Utc::now()),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(NewsletterSubscribersResponse {
            items: vec![NewsletterSubscriberResponse {
                subscriber_id: inserted.subscriber_id,
                email: inserted.email,
                subscription_date: inserted.subscription_date.to_rfc3339(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
