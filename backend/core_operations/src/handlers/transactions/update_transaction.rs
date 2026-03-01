use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::transactions;
use proto::proto::core::{TransactionResponse, TransactionsResponse, UpdateTransactionRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_transaction(
    txn: &DatabaseTransaction,
    request: Request<UpdateTransactionRequest>,
) -> Result<Response<TransactionsResponse>, Status> {
    let req = request.into_inner();

    let existing = transactions::Entity::find_by_id(req.transaction_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!(
                "Transaction with ID {} not found",
                req.transaction_id
            ))
        })?;

    let amount = req
        .amount_paise
        .map(paise_to_decimal)
        .unwrap_or(existing.amount);

    let model = transactions::ActiveModel {
        transaction_id: ActiveValue::Set(existing.transaction_id),
        user_id: ActiveValue::Set(req.user_id.unwrap_or(existing.user_id)),
        amount: ActiveValue::Set(amount),
        transaction_date: ActiveValue::Set(existing.transaction_date),
        r#type: ActiveValue::Set(req.r#type.unwrap_or(existing.r#type)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(TransactionsResponse {
            items: vec![TransactionResponse {
                transaction_id: updated.transaction_id,
                user_id: updated.user_id,
                amount_paise: decimal_to_paise(&updated.amount),
                transaction_date: updated.transaction_date.to_rfc3339(),
                r#type: updated.r#type,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
