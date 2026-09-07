//! Admin: list/search coupons.

use crate::handlers::coupons::create_coupon::model_to_admin_response;
use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::coupons;
use proto::proto::core::{CouponsAdminResponse, SearchCouponAdminRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_coupon_admin(
    txn: &DatabaseTransaction,
    request: Request<SearchCouponAdminRequest>,
) -> Result<Response<CouponsAdminResponse>, Status> {
    let req = request.into_inner();

    let mut query = coupons::Entity::find();
    if req.coupon_id != 0 {
        query = query.filter(coupons::Column::CouponId.eq(req.coupon_id));
    }

    let models = query.all(txn).await.map_err(map_db_error_to_status)?;
    Ok(Response::new(CouponsAdminResponse {
        items: models.into_iter().map(model_to_admin_response).collect(),
    }))
}
