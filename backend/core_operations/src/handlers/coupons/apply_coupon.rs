use crate::handlers::coupons::validate_coupon::check_coupon;
use proto::proto::core::{ApplyCouponRequest, CouponsResponse};
use sea_orm::DatabaseTransaction;
use tonic::{Request, Response, Status};

pub async fn apply_coupon(
    txn: &DatabaseTransaction,
    request: Request<ApplyCouponRequest>,
) -> Result<Response<CouponsResponse>, Status> {
    let req = request.into_inner();

    let result = check_coupon(txn, &req.code, req.order_amount_paise, false).await?;

    Ok(Response::new(CouponsResponse {
        items: vec![result],
    }))
}
