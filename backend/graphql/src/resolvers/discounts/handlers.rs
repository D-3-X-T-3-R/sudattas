use proto::proto::core::{
    CreateDiscountRequest, DeleteDiscountRequest, DiscountResponse, SearchDiscountRequest,
    UpdateDiscountRequest,
};
use tracing::instrument;

use super::schema::{Discount, DiscountMutation, NewDiscount, SearchDiscount};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn discount_response_to_gql(d: DiscountResponse) -> Discount {
    Discount {
        discount_id: d.discount_id.to_string(),
        product_id: d.product_id.to_string(),
        discount_percentage: d.discount_percentage,
        start_date: d.start_date,
        end_date: d.end_date,
    }
}

#[instrument]
pub(crate) async fn search_discount(input: SearchDiscount) -> Result<Vec<Discount>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .search_discount(SearchDiscountRequest {
            discount_id: input
                .discount_id
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(discount_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn create_discount(input: NewDiscount) -> Result<Vec<Discount>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_discount(CreateDiscountRequest {
            product_id: parse_i64(&input.product_id, "product id")?,
            discount_percentage: input.discount_percentage,
            start_date: input.start_date,
            end_date: input.end_date,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(discount_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_discount(input: DiscountMutation) -> Result<Vec<Discount>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_discount(UpdateDiscountRequest {
            discount_id: parse_i64(&input.discount_id, "discount id")?,
            product_id: input
                .product_id
                .as_deref()
                .map(|s| parse_i64(s, "product id"))
                .transpose()?,
            discount_percentage: input.discount_percentage,
            start_date: input.start_date,
            end_date: input.end_date,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(discount_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_discount(discount_id: String) -> Result<Vec<Discount>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .delete_discount(DeleteDiscountRequest {
            discount_id: parse_i64(&discount_id, "discount id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(discount_response_to_gql)
        .collect())
}
