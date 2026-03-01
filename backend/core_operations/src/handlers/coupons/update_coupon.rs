//! P1 Admin: update coupon (disable, set limits/expiry).

use crate::handlers::coupons::create_coupon::model_to_admin_response;
use crate::handlers::db_errors::map_db_error_to_status;
use chrono::{DateTime, Utc};
use core_db_entities::entity::coupons;
use core_db_entities::entity::sea_orm_active_enums::CouponStatus;
use proto::proto::core::{CouponAdminResponse, CouponsAdminResponse, UpdateCouponRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait, IntoActiveModel};
use tonic::{Request, Response, Status};

pub async fn update_coupon(
    txn: &DatabaseTransaction,
    request: Request<UpdateCouponRequest>,
) -> Result<Response<CouponsAdminResponse>, Status> {
    let req = request.into_inner();

    let existing = coupons::Entity::find_by_id(req.coupon_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("Coupon {} not found", req.coupon_id)))?;

    let mut active: coupons::ActiveModel = existing.into_active_model();

    if let Some(ref s) = req.status {
        let status = match s.to_lowercase().as_str() {
            "active" => CouponStatus::Active,
            "inactive" => CouponStatus::Inactive,
            _ => {
                return Err(Status::invalid_argument(
                    "status must be active or inactive",
                ))
            }
        };
        active.coupon_status = ActiveValue::Set(Some(status));
    }
    if req.usage_limit.is_some() {
        active.usage_limit = ActiveValue::Set(req.usage_limit);
    }
    if let Some(ref s) = req.ends_at {
        if s.is_empty() {
            active.ends_at = ActiveValue::Set(None);
        } else {
            let dt = DateTime::parse_from_rfc3339(s)
                .map_err(|_| Status::invalid_argument("ends_at must be RFC3339"))?
                .with_timezone(&Utc);
            active.ends_at = ActiveValue::Set(Some(dt));
        }
    }

    let updated = active.update(txn).await.map_err(map_db_error_to_status)?;
    Ok(Response::new(CouponsAdminResponse {
        items: vec![model_to_admin_response(updated)],
    }))
}
