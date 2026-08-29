//! Public unsubscribe link clicked from inside a campaign email — no admin/customer auth,
//! verified purely by the signed token (see `unsubscribe_token.rs`).

use super::unsubscribe_token::verify_unsubscribe_token;
use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::newsletter_subscribers;
use proto::proto::core::{
    NewsletterSubscriberResponse, NewsletterSubscribersResponse, UnsubscribeNewsletterByTokenRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait, IntoActiveModel};
use tonic::{Request, Response, Status};

pub async fn unsubscribe_newsletter_by_token(
    txn: &DatabaseTransaction,
    request: Request<UnsubscribeNewsletterByTokenRequest>,
) -> Result<Response<NewsletterSubscribersResponse>, Status> {
    let req = request.into_inner();

    if !verify_unsubscribe_token(req.subscriber_id, &req.token) {
        return Err(Status::permission_denied("Invalid or expired unsubscribe link"));
    }

    let existing = newsletter_subscribers::Entity::find_by_id(req.subscriber_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Subscriber not found"))?;

    // Idempotent: clicking an already-processed unsubscribe link (e.g. opened twice) should
    // not error, and should not overwrite the original unsubscribe timestamp.
    if existing.unsubscribed_at.is_some() {
        return Ok(Response::new(NewsletterSubscribersResponse {
            items: vec![NewsletterSubscriberResponse {
                subscriber_id: existing.subscriber_id,
                email: existing.email,
                subscription_date: existing.subscription_date.to_rfc3339(),
                unsubscribed_at: existing
                    .unsubscribed_at
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default(),
            }],
        }));
    }

    let mut active = existing.into_active_model();
    active.unsubscribed_at = ActiveValue::Set(Some(Utc::now()));

    let updated = active.update(txn).await.map_err(map_db_error_to_status)?;

    Ok(Response::new(NewsletterSubscribersResponse {
        items: vec![NewsletterSubscriberResponse {
            subscriber_id: updated.subscriber_id,
            email: updated.email,
            subscription_date: updated.subscription_date.to_rfc3339(),
            unsubscribed_at: updated
                .unsubscribed_at
                .map(|v| v.to_rfc3339())
                .unwrap_or_default(),
        }],
    }))
}
