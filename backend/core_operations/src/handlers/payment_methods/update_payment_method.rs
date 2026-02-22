use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_methods;
use proto::proto::core::{
    PaymentMethodResponse, PaymentMethodsResponse, UpdatePaymentMethodRequest,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_payment_method(
    txn: &DatabaseTransaction,
    request: Request<UpdatePaymentMethodRequest>,
) -> Result<Response<PaymentMethodsResponse>, Status> {
    let req = request.into_inner();

    // Load existing entity
    let existing = payment_methods::Entity::find_by_id(req.method_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Payment method not found"))?;

    #[allow(clippy::unnecessary_map_or)]
    let model = payment_methods::ActiveModel {
        method_id: ActiveValue::Set(existing.method_id),
        method_name: if req.method_name.as_ref().map_or(true, |s| s.is_empty()) {
            ActiveValue::Set(existing.method_name)
        } else {
            ActiveValue::Set(req.method_name.unwrap())
        },
        details: if req.details.as_ref().map_or(true, |s| s.is_empty()) {
            ActiveValue::Set(existing.details)
        } else {
            ActiveValue::Set(Some(req.details.unwrap()))
        },
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(PaymentMethodsResponse {
            items: vec![PaymentMethodResponse {
                method_id: updated.method_id,
                method_name: updated.method_name,
                details: updated.details.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
