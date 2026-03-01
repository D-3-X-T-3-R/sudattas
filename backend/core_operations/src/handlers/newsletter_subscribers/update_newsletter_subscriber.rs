use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::newsletter_subscribers;
use proto::proto::core::{
    NewsletterSubscriberResponse, NewsletterSubscribersResponse, UpdateNewsletterSubscriberRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_newsletter_subscriber(
    txn: &DatabaseTransaction,
    request: Request<UpdateNewsletterSubscriberRequest>,
) -> Result<Response<NewsletterSubscribersResponse>, Status> {
    let req = request.into_inner();

    let existing = newsletter_subscribers::Entity::find_by_id(req.subscriber_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "NewsletterSubscriber with ID {} not found",
                req.subscriber_id
            ))
        })?;

    let model = newsletter_subscribers::ActiveModel {
        subscriber_id: ActiveValue::Set(existing.subscriber_id),
        email: ActiveValue::Set(req.email),
        subscription_date: ActiveValue::Set(existing.subscription_date),
        unsubscribed_at: ActiveValue::Set(existing.unsubscribed_at),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(NewsletterSubscribersResponse {
            items: vec![NewsletterSubscriberResponse {
                subscriber_id: updated.subscriber_id,
                email: updated.email,
                subscription_date: updated.subscription_date.to_rfc3339(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
