use proto::proto::core::{
    CreateNewsletterSubscriberRequest, DeleteNewsletterSubscriberRequest,
    NewsletterSubscriberResponse, NewsletterSubscribersResponse, SearchNewsletterSubscriberRequest,
    UpdateNewsletterSubscriberRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteNewsletterSubscriberInput, NewNewsletterSubscriber, NewsletterSubscriber,
    NewsletterSubscriberMutation, SearchNewsletterSubscriberInput,
};
use crate::resolvers::{error::GqlError, utils::connect_grpc_client};

fn sub_response_to_gql(s: NewsletterSubscriberResponse) -> NewsletterSubscriber {
    NewsletterSubscriber {
        subscriber_id: s.subscriber_id.to_string(),
        email: s.email,
        subscription_date: s.subscription_date,
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
