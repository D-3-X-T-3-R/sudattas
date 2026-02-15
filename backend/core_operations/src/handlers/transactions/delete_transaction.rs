use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::transactions;
use proto::proto::core::{DeleteTransactionRequest, TransactionResponse, TransactionsResponse};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_transaction(
    txn: &DatabaseTransaction,
    request: Request<DeleteTransactionRequest>,
) -> Result<Response<TransactionsResponse>, Status> {
    let req = request.into_inner();

    let found = transactions::Entity::find_by_id(req.transaction_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match transactions::Entity::delete_by_id(req.transaction_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(TransactionsResponse {
                    items: vec![TransactionResponse {
                        transaction_id: model.transaction_id,
                        user_id: model.user_id,
                        amount: model.amount.to_f64().unwrap_or(0.0),
                        transaction_date: model.transaction_date.to_rfc3339(),
                        r#type: model.r#type,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Transaction with ID {} not found",
            req.transaction_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
