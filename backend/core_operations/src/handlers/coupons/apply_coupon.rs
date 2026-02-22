use crate::handlers::coupons::validate_coupon::check_coupon;
use core_db_entities::entity::coupons;
use proto::proto::core::{ApplyCouponRequest, CouponsResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel,
    QueryFilter,
};
use tonic::{Request, Response, Status};

pub async fn apply_coupon(
    txn: &DatabaseTransaction,
    request: Request<ApplyCouponRequest>,
) -> Result<Response<CouponsResponse>, Status> {
    let req = request.into_inner();

    let result = check_coupon(txn, &req.code, req.order_amount_paise, true).await?;

    if result.is_valid {
        // Increment usage_count
        if let Some(coupon) = coupons::Entity::find()
            .filter(coupons::Column::Code.eq(&req.code))
            .one(txn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
        {
            let mut active = coupon.into_active_model();
            let new_count = active
                .usage_count
                .take()
                .flatten()
                .map(|c| c + 1)
                .unwrap_or(1);
            active.usage_count = ActiveValue::Set(Some(new_count));
            active
                .update(txn)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
        }
    }

    Ok(Response::new(CouponsResponse { items: vec![result] }))
}
