use proto::proto::core::{ApplyCouponRequest, CouponResponse, ValidateCouponRequest};
use tracing::instrument;

use super::schema::{ApplyCoupon, Coupon, ValidateCoupon};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn coupon_response_to_gql(c: CouponResponse) -> Coupon {
    Coupon {
        coupon_id: c.coupon_id.to_string(),
        code: c.code,
        discount_type: c.discount_type,
        discount_value: c.discount_value,
        discount_amount_paise: c.discount_amount_paise.to_string(),
        final_amount_paise: c.final_amount_paise.to_string(),
        is_valid: c.is_valid,
        reason: c.reason,
    }
}

#[instrument]
pub(crate) async fn validate_coupon(input: ValidateCoupon) -> Result<Vec<Coupon>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .validate_coupon(ValidateCouponRequest {
            code: input.code,
            order_amount_paise: parse_i64(&input.order_amount_paise, "order_amount_paise")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(coupon_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn apply_coupon(input: ApplyCoupon) -> Result<Vec<Coupon>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .apply_coupon(ApplyCouponRequest {
            code: input.code,
            order_amount_paise: parse_i64(&input.order_amount_paise, "order_amount_paise")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(coupon_response_to_gql)
        .collect())
}
