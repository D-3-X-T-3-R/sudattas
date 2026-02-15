use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_methods;
use proto::proto::core::{
    DeletePaymentMethodRequest, PaymentMethodResponse, PaymentMethodsResponse,
};
use sea_orm::{ActiveModelTrait, DatabaseTransaction, EntityTrait, IntoActiveModel};
use tonic::{Request, Response, Status};

pub async fn delete_payment_method(
    txn: &DatabaseTransaction,
    request: Request<DeletePaymentMethodRequest>,
) -> Result<Response<PaymentMethodsResponse>, Status> {
    let req = request.into_inner();

    // Load existing entity
    let existing = payment_methods::Entity::find_by_id(req.method_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found("Payment method not found"))?;

    let method_response = PaymentMethodResponse {
        method_id: existing.method_id,
        method_name: existing.method_name.clone(),
        details: existing.details.clone().unwrap_or_default(),
    };

    // Delete the entity
    existing
        .into_active_model()
        .delete(txn)
        .await
        .map_err(map_db_error_to_status)?;

    Ok(Response::new(PaymentMethodsResponse {
        items: vec![method_response],
    }))
}
