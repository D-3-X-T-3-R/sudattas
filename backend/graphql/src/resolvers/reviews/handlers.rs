use proto::proto::core::{
    CreateReviewRequest, DeleteReviewRequest, ReviewResponse, SearchReviewRequest,
    UpdateReviewRequest,
};
use tracing::instrument;

use super::schema::{NewReview, Review, ReviewMutation, SearchReview};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_option_i64},
};

fn review_response_to_gql(r: ReviewResponse) -> Review {
    Review {
        review_id: r.review_id.to_string(),
        product_id: r.product_id.to_string(),
        user_id: r.user_id.to_string(),
        rating: r.rating as i32,
        comment: r.comment,
    }
}

#[instrument]
pub(crate) async fn create_review(input: NewReview) -> Result<Vec<Review>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_review(CreateReviewRequest {
            product_id: parse_i64(&input.product_id, "product id")?,
            user_id: parse_i64(&input.user_id, "user id")?,
            rating: input.rating as i64,
            comment: input.comment,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(review_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_review(input: SearchReview) -> Result<Vec<Review>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let review_id = input
        .review_id
        .as_deref()
        .map(|s| parse_i64(s, "review id"))
        .transpose()?
        .unwrap_or(0);
    let response = client
        .search_review(SearchReviewRequest {
            review_id,
            product_id: to_option_i64(input.product_id),
            user_id: to_option_i64(input.user_id),
            limit: crate::graphql_limits::cap_page_size(to_option_i64(input.limit)),
            offset: to_option_i64(input.offset),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(review_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_review(input: ReviewMutation) -> Result<Vec<Review>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_review(UpdateReviewRequest {
            review_id: parse_i64(&input.review_id, "review id")?,
            product_id: input
                .product_id
                .as_deref()
                .map(|s| parse_i64(s, "product id"))
                .transpose()?,
            user_id: input
                .user_id
                .as_deref()
                .map(|s| parse_i64(s, "user id"))
                .transpose()?,
            rating: input.rating.map(|r| r as i64),
            comment: input.comment,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(review_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_review(review_id: String) -> Result<Vec<Review>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .delete_review(DeleteReviewRequest {
            review_id: parse_i64(&review_id, "review id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(review_response_to_gql)
        .collect())
}
