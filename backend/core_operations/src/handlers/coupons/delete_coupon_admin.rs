//! Admin: delete a coupon.

use crate::handlers::coupons::create_coupon::model_to_admin_response;
use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::coupons;
use proto::proto::core::{CouponsAdminResponse, DeleteCouponAdminRequest};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_coupon_admin(
    txn: &DatabaseTransaction,
    request: Request<DeleteCouponAdminRequest>,
) -> Result<Response<CouponsAdminResponse>, Status> {
    let req = request.into_inner();

    let found = coupons::Entity::find_by_id(req.coupon_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("Coupon {} not found", req.coupon_id)))?;

    coupons::Entity::delete_by_id(req.coupon_id)
        .exec(txn)
        .await
        .map_err(map_db_error_to_status)?;

    Ok(Response::new(CouponsAdminResponse {
        items: vec![model_to_admin_response(found)],
    }))
}
