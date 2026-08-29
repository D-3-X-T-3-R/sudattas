use proto::proto::core::{
    CreateNewsletterSubscriberRequest, DeleteNewsletterSubscriberRequest,
    NewsletterSubscriberResponse, NewsletterSubscribersResponse, SearchNewsletterCampaignRequest,
    SearchNewsletterSubscriberRequest, SendNewsletterCampaignRequest,
    UnsubscribeNewsletterByTokenRequest, UpdateNewsletterSubscriberRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteNewsletterSubscriberInput, NewNewsletterSubscriber, NewsletterCampaign,
    NewsletterSubscriber, NewsletterSubscriberMutation, SearchNewsletterCampaignInput,
    SearchNewsletterSubscriberInput, SendNewsletterCampaignInput, UnsubscribeNewsletterInput,
};
use crate::resolvers::{error::GqlError, utils::connect_grpc_client};

fn sub_response_to_gql(s: NewsletterSubscriberResponse) -> NewsletterSubscriber {
    NewsletterSubscriber {
        subscriber_id: s.subscriber_id.to_string(),
        email: s.email,
        subscription_date: s.subscription_date,
        unsubscribed_at: s.unsubscribed_at,
    }
}

fn subs_response_to_vec(resp: NewsletterSubscribersResponse) -> Vec<NewsletterSubscriber> {
    resp.items.into_iter().map(sub_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_newsletter_subscriber(
    input: NewNewsletterSubscriber,
) -> Result<Vec<NewsletterSubscriber>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_newsletter_subscriber(CreateNewsletterSubscriberRequest { email: input.email })
        .await?;
    Ok(subs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_newsletter_subscriber(
    input: SearchNewsletterSubscriberInput,
) -> Result<Vec<NewsletterSubscriber>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_newsletter_subscriber(SearchNewsletterSubscriberRequest {
            subscriber_id: crate::resolvers::utils::to_i64(Some(input.subscriber_id)),
        })
        .await?;
    Ok(subs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_newsletter_subscriber(
    input: NewsletterSubscriberMutation,
) -> Result<Vec<NewsletterSubscriber>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_newsletter_subscriber(UpdateNewsletterSubscriberRequest {
            subscriber_id: crate::resolvers::utils::to_i64(Some(input.subscriber_id)),
            email: input.email,
            unsubscribed: input.unsubscribed,
        })
        .await?;
    Ok(subs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_newsletter_subscriber(
    input: DeleteNewsletterSubscriberInput,
) -> Result<Vec<NewsletterSubscriber>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_newsletter_subscriber(DeleteNewsletterSubscriberRequest {
            subscriber_id: crate::resolvers::utils::to_i64(Some(input.subscriber_id)),
        })
        .await?;
    Ok(subs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn unsubscribe_newsletter(
    input: UnsubscribeNewsletterInput,
) -> Result<Vec<NewsletterSubscriber>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .unsubscribe_newsletter_by_token(UnsubscribeNewsletterByTokenRequest {
            subscriber_id: crate::resolvers::utils::to_i64(Some(input.subscriber_id)),
            token: input.token,
        })
        .await?;
    Ok(subs_response_to_vec(resp.into_inner()))
}

fn campaign_response_to_gql(c: proto::proto::core::NewsletterCampaignResponse) -> NewsletterCampaign {
    NewsletterCampaign {
        campaign_id: c.campaign_id.to_string(),
        subject: c.subject,
        body_text: c.body_text,
        cta_label: c.cta_label,
        cta_url: c.cta_url,
        recipient_count: c.recipient_count.to_string(),
        success_count: c.success_count.to_string(),
        failure_count: c.failure_count.to_string(),
        sent_at: c.sent_at,
    }
}

#[instrument]
pub(crate) async fn send_newsletter_campaign(
    input: SendNewsletterCampaignInput,
) -> Result<Vec<NewsletterCampaign>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .send_newsletter_campaign(SendNewsletterCampaignRequest {
            subject: input.subject,
            body_text: input.body_text,
            cta_label: input.cta_label,
            cta_url: input.cta_url,
        })
        .await?;
    Ok(resp
        .into_inner()
        .items
        .into_iter()
        .map(campaign_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_newsletter_campaign(
    input: SearchNewsletterCampaignInput,
) -> Result<Vec<NewsletterCampaign>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_newsletter_campaign(SearchNewsletterCampaignRequest {
            campaign_id: input
                .campaign_id
                .map(|s| crate::resolvers::utils::to_i64(Some(s))),
        })
        .await?;
    Ok(resp
        .into_inner()
        .items
        .into_iter()
        .map(campaign_response_to_gql)
        .collect())
}
