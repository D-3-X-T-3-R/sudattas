use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::payment_methods;
use proto::proto::core::{
    SearchPaymentMethodRequest, PaymentMethodResponse, PaymentMethodsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_payment_method(
    txn: &DatabaseTransaction,
    request: Request<SearchPaymentMethodRequest>,
) -> Result<Response<PaymentMethodsResponse>, Status> {
    let req = request.into_inner();

    let mut query = payment_methods::Entity::find();
    if req.method_id != 0 {
        query = query.filter(payment_methods::Column::MethodId.eq(req.method_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| PaymentMethodResponse {
                    method_id: m.method_id,
                    method_name: m.method_name,
                    details: m.details.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(PaymentMethodsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
