use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::newsletter_campaigns;
use proto::proto::core::{
    NewsletterCampaignResponse, NewsletterCampaignsResponse, SearchNewsletterCampaignRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use tonic::{Request, Response, Status};

pub async fn search_newsletter_campaign(
    txn: &DatabaseTransaction,
    request: Request<SearchNewsletterCampaignRequest>,
) -> Result<Response<NewsletterCampaignsResponse>, Status> {
    let req = request.into_inner();

    let mut query =
        newsletter_campaigns::Entity::find().order_by_desc(newsletter_campaigns::Column::SentAt);
    if let Some(id) = req.campaign_id {
        query = query.filter(newsletter_campaigns::Column::CampaignId.eq(id));
    }

    let models = query.all(txn).await.map_err(map_db_error_to_status)?;
    let items = models
        .into_iter()
        .map(|m| NewsletterCampaignResponse {
            campaign_id: m.campaign_id,
            subject: m.subject,
            body_text: m.body_text,
            cta_label: m.cta_label.unwrap_or_default(),
            cta_url: m.cta_url.unwrap_or_default(),
            recipient_count: m.recipient_count as i64,
            success_count: m.success_count as i64,
            failure_count: m.failure_count as i64,
            sent_at: m.sent_at.to_rfc3339(),
        })
        .collect();

    Ok(Response::new(NewsletterCampaignsResponse { items }))
}
