use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::newsletter_subscribers;
use proto::proto::core::{
    NewsletterSubscriberResponse, NewsletterSubscribersResponse,
    SearchNewsletterSubscriberRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_newsletter_subscriber(
    txn: &DatabaseTransaction,
    request: Request<SearchNewsletterSubscriberRequest>,
) -> Result<Response<NewsletterSubscribersResponse>, Status> {
    let req = request.into_inner();

    let mut query = newsletter_subscribers::Entity::find();
    if req.subscriber_id != 0 {
        query = query.filter(newsletter_subscribers::Column::SubscriberId.eq(req.subscriber_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| NewsletterSubscriberResponse {
                    subscriber_id: m.subscriber_id,
                    email: m.email,
                    subscription_date: m.subscription_date.to_rfc3339(),
                })
                .collect();
            Ok(Response::new(NewsletterSubscribersResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
