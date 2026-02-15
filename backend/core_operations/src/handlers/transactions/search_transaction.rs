use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::transactions;
use proto::proto::core::{SearchTransactionRequest, TransactionResponse, TransactionsResponse};
use rust_decimal::prelude::ToPrimitive;
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_transaction(
    txn: &DatabaseTransaction,
    request: Request<SearchTransactionRequest>,
) -> Result<Response<TransactionsResponse>, Status> {
    let req = request.into_inner();

    let mut query = transactions::Entity::find();
    if req.transaction_id != 0 {
        query = query.filter(transactions::Column::TransactionId.eq(req.transaction_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| TransactionResponse {
                    transaction_id: m.transaction_id,
                    user_id: m.user_id,
                    amount: m.amount.to_f64().unwrap_or(0.0),
                    transaction_date: m.transaction_date.to_rfc3339(),
                    r#type: m.r#type.clone(),
                })
                .collect();
            Ok(Response::new(TransactionsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
