use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use chrono::Utc;
use core_db_entities::entity::transactions;
use proto::proto::core::{CreateTransactionRequest, TransactionResponse, TransactionsResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_transaction(
    txn: &DatabaseTransaction,
    request: Request<CreateTransactionRequest>,
) -> Result<Response<TransactionsResponse>, Status> {
    let req = request.into_inner();
    let model = transactions::ActiveModel {
        transaction_id: ActiveValue::NotSet,
        user_id: ActiveValue::Set(req.user_id),
        amount: ActiveValue::Set(paise_to_decimal(req.amount_paise)),
        transaction_date: ActiveValue::Set(Utc::now()),
        r#type: ActiveValue::Set(req.r#type),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(TransactionsResponse {
            items: vec![TransactionResponse {
                transaction_id: inserted.transaction_id,
                user_id: inserted.user_id,
                amount_paise: decimal_to_paise(&inserted.amount),
                transaction_date: inserted.transaction_date.to_rfc3339(),
                r#type: inserted.r#type,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
