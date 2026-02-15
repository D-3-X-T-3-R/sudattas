use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_methods;
use proto::proto::core::{
    CreatePaymentMethodRequest, PaymentMethodResponse, PaymentMethodsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_payment_method(
    txn: &DatabaseTransaction,
    request: Request<CreatePaymentMethodRequest>,
) -> Result<Response<PaymentMethodsResponse>, Status> {
    let req = request.into_inner();
    let model = payment_methods::ActiveModel {
        method_id: ActiveValue::NotSet,
        method_name: ActiveValue::Set(req.method_name),
        details: ActiveValue::Set(Some(req.details)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(PaymentMethodsResponse {
            items: vec![PaymentMethodResponse {
                method_id: inserted.method_id,
                method_name: inserted.method_name,
                details: inserted.details.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
