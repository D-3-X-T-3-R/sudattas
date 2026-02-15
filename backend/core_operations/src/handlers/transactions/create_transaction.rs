use crate::handlers::db_errors::map_db_error_to_status;
use chrono::Utc;
use core_db_entities::entity::transactions;
use proto::proto::core::{CreateTransactionRequest, TransactionResponse, TransactionsResponse};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_transaction(
    txn: &DatabaseTransaction,
    request: Request<CreateTransactionRequest>,
) -> Result<Response<TransactionsResponse>, Status> {
    let req = request.into_inner();
    let amount = Decimal::from_f64_retain(req.amount).unwrap_or(Decimal::ZERO);
    let model = transactions::ActiveModel {
        transaction_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(req.user_id),
        amount: ActiveValue::Set(amount),
        transaction_date: ActiveValue::Set(Utc::now()),
        r#type: ActiveValue::Set(req.r#type),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(TransactionsResponse {
            items: vec![TransactionResponse {
                transaction_id: inserted.transaction_id,
                user_id: inserted.user_id,
                amount: inserted.amount.to_f64().unwrap_or(0.0),
                transaction_date: inserted.transaction_date.to_rfc3339(),
                r#type: inserted.r#type,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
