use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::newsletter_subscribers;
use proto::proto::core::{
    DeleteNewsletterSubscriberRequest, NewsletterSubscriberResponse, NewsletterSubscribersResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_newsletter_subscriber(
    txn: &DatabaseTransaction,
    request: Request<DeleteNewsletterSubscriberRequest>,
) -> Result<Response<NewsletterSubscribersResponse>, Status> {
    let req = request.into_inner();

    let found = newsletter_subscribers::Entity::find_by_id(req.subscriber_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match newsletter_subscribers::Entity::delete_by_id(req.subscriber_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(NewsletterSubscribersResponse {
                    items: vec![NewsletterSubscriberResponse {
                        subscriber_id: model.subscriber_id,
                        email: model.email,
                        subscription_date: model.subscription_date.to_rfc3339(),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "NewsletterSubscriber with ID {} not found",
            req.subscriber_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
