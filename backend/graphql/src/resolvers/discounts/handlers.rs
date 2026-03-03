//! Discount resolvers: gRPC Discount APIs were removed; schema kept for compatibility, returns empty.

use tracing::instrument;

use super::schema::{Discount, DiscountMutation, NewDiscount, SearchDiscount};
use crate::resolvers::error::GqlError;

#[instrument]
pub(crate) async fn search_discount(_input: SearchDiscount) -> Result<Vec<Discount>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn create_discount(_input: NewDiscount) -> Result<Vec<Discount>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn update_discount(_input: DiscountMutation) -> Result<Vec<Discount>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn delete_discount(_discount_id: String) -> Result<Vec<Discount>, GqlError> {
    Ok(vec![])
}
