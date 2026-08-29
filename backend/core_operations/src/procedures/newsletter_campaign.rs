//! Admin: compose and immediately send a newsletter campaign to every subscriber who hasn't
//! unsubscribed. No draft/scheduled state — this either sends and records what happened, or
//! fails before sending anything. Phased like `place_order` (read subscribers -> external
//! Resend calls with no DB transaction held -> one final write), since the send loop is the
//! only slow part and must never hold a pooled DB connection idle for its duration.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::handlers::newsletter_subscribers::unsubscribe_token::generate_unsubscribe_token;
use crate::notifications::email_provider::send_transactional_email;
use crate::notifications::brand_email::storefront_url;
use crate::notifications::newsletter_mail::build_newsletter_campaign_email;
use chrono::Utc;
use core_db_entities::entity::{newsletter_campaigns, newsletter_subscribers};
use proto::proto::core::{
    NewsletterCampaignResponse, NewsletterCampaignsResponse, SendNewsletterCampaignRequest,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tonic::{Request, Response, Status};
use tracing::warn;

/// Safety cap so a runaway subscriber list (or a mistaken repeated click) can't fire an
/// unbounded number of external API calls from one request.
const MAX_RECIPIENTS_PER_SEND: usize = 5000;

/// Delay between individual Resend calls so a large subscriber list doesn't burst past
/// Resend's per-second rate limit. Deliberately conservative — this is a low-frequency admin
/// action, not a latency-sensitive one.
const SEND_DELAY: std::time::Duration = std::time::Duration::from_millis(350);

pub async fn send_newsletter_campaign(
    db: &DatabaseConnection,
    request: Request<SendNewsletterCampaignRequest>,
) -> Result<Response<NewsletterCampaignsResponse>, Status> {
    let req = request.into_inner();

    let subject = req.subject.trim().to_string();
    if subject.is_empty() {
        return Err(Status::invalid_argument("subject is required"));
    }
    let body_text = req.body_text.trim().to_string();
    if body_text.is_empty() {
        return Err(Status::invalid_argument("body_text is required"));
    }
    let cta_label = req
        .cta_label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    let cta_url = req
        .cta_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from);
    if cta_label.is_some() != cta_url.is_some() {
        return Err(Status::invalid_argument(
            "cta_label and cta_url must both be set, or both omitted",
        ));
    }

    let subscribers = newsletter_subscribers::Entity::find()
        .filter(newsletter_subscribers::Column::UnsubscribedAt.is_null())
        .all(db)
        .await
        .map_err(map_db_error_to_status)?;

    if subscribers.is_empty() {
        return Err(Status::failed_precondition(
            "No active (non-unsubscribed) subscribers to send to",
        ));
    }
    if subscribers.len() > MAX_RECIPIENTS_PER_SEND {
        return Err(Status::failed_precondition(format!(
            "{} active subscribers exceeds the per-send cap of {}",
            subscribers.len(),
            MAX_RECIPIENTS_PER_SEND
        )));
    }

    let recipient_count = subscribers.len();
    let site_url = storefront_url();
    let mut success_count: i32 = 0;
    let mut failure_count: i32 = 0;

    for (idx, sub) in subscribers.iter().enumerate() {
        if idx > 0 {
            tokio::time::sleep(SEND_DELAY).await;
        }

        let Some(token) = generate_unsubscribe_token(sub.subscriber_id) else {
            // Misconfigured INTERNAL_API_SECRET — don't send an email with a broken/absent
            // unsubscribe link; count it as a failure and move on to the next subscriber.
            warn!(
                subscriber_id = sub.subscriber_id,
                "newsletter campaign: could not build unsubscribe token (INTERNAL_API_SECRET not set); skipping recipient"
            );
            failure_count += 1;
            continue;
        };
        let unsubscribe_url = format!(
            "{}/newsletter/unsubscribe?id={}&token={}",
            site_url.trim_end_matches('/'),
            sub.subscriber_id,
            token
        );

        let (text, html) = build_newsletter_campaign_email(
            &subject,
            &body_text,
            cta_label.as_deref(),
            cta_url.as_deref(),
            &unsubscribe_url,
        );

        match send_transactional_email(&sub.email, &subject, &text, &html).await {
            Ok(()) => success_count += 1,
            Err(e) => {
                warn!(
                    subscriber_id = sub.subscriber_id,
                    error = %e,
                    "newsletter campaign: send failed for one recipient"
                );
                failure_count += 1;
            }
        }
    }

    let model = newsletter_campaigns::ActiveModel {
        campaign_id: ActiveValue::NotSet,
        subject: ActiveValue::Set(subject),
        body_text: ActiveValue::Set(body_text),
        cta_label: ActiveValue::Set(cta_label),
        cta_url: ActiveValue::Set(cta_url),
        recipient_count: ActiveValue::Set(recipient_count as i32),
        success_count: ActiveValue::Set(success_count),
        failure_count: ActiveValue::Set(failure_count),
        sent_at: ActiveValue::Set(Utc::now()),
    };
    let inserted = model.insert(db).await.map_err(map_db_error_to_status)?;

    Ok(Response::new(NewsletterCampaignsResponse {
        items: vec![NewsletterCampaignResponse {
            campaign_id: inserted.campaign_id,
            subject: inserted.subject,
            body_text: inserted.body_text,
            cta_label: inserted.cta_label.unwrap_or_default(),
            cta_url: inserted.cta_url.unwrap_or_default(),
            recipient_count: inserted.recipient_count as i64,
            success_count: inserted.success_count as i64,
            failure_count: inserted.failure_count as i64,
            sent_at: inserted.sent_at.to_rfc3339(),
        }],
    }))
}
