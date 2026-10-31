use proto::proto::core::{
    AdminUpdateReviewStatusRequest, CreateReviewRequest, DeleteReviewRequest,
    ProductRatingSummaryRequest, ReviewResponse, SearchReviewRequest, UpdateReviewRequest,
};
use tracing::instrument;

use super::schema::{
    AdminUpdateReviewStatusInput, NewReview, ProductRatingSummary, Review, ReviewMutation,
    SearchReview,
};
use crate::{
    resolvers::{
        error::GqlError,
        utils::{connect_grpc_client, parse_i64, to_option_i64},
    },
    validation,
};

fn review_response_to_gql(r: ReviewResponse) -> Review {
    Review {
        review_id: r.review_id.to_string(),
        product_id: r.product_id.to_string(),
        user_id: r.user_id.to_string(),
        rating: r.rating,
        comment: r.comment,
        review_status: r.review_status,
        is_verified_purchase: r.is_verified_purchase,
        created_at: r.created_at,
    }
}

#[instrument]
pub(crate) async fn create_review(input: NewReview) -> Result<Vec<Review>, GqlError> {
    validation::validate_rating(input.rating)?;
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_review(CreateReviewRequest {
            product_id: parse_i64(&input.product_id, "product id")?,
            user_id: parse_i64(&input.user_id, "user id")?,
            rating: input.rating,
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
            status_filter: input.status_filter,
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
    if let Some(rating) = input.rating {
        validation::validate_rating(rating)?;
    }
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
            rating: input.rating,
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

#[instrument]
pub(crate) async fn get_product_rating_summary(
    product_id: String,
) -> Result<ProductRatingSummary, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_product_rating_summary(ProductRatingSummaryRequest {
            product_id: parse_i64(&product_id, "product id")?,
        })
        .await?;
    let r = response.into_inner();
    Ok(ProductRatingSummary {
        product_id: r.product_id.to_string(),
        average_rating: r.average_rating,
        rating_count: r.rating_count as i32,
    })
}

#[instrument]
pub(crate) async fn admin_update_review_status(
    input: AdminUpdateReviewStatusInput,
) -> Result<bool, GqlError> {
    let mut client = connect_grpc_client().await?;
    let _ = client
        .admin_update_review_status(AdminUpdateReviewStatusRequest {
            review_id: parse_i64(&input.review_id, "review id")?,
            status: input.status,
        })
        .await?;
    Ok(true)
}
