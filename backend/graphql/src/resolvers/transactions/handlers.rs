use proto::proto::core::{
    CreateTransactionRequest, DeleteTransactionRequest, SearchTransactionRequest,
    TransactionResponse, TransactionsResponse, UpdateTransactionRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteTransactionInput, NewTransaction, SearchTransactionInput, Transaction,
    TransactionMutation,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn tx_response_to_gql(t: TransactionResponse) -> Transaction {
    Transaction {
        transaction_id: t.transaction_id.to_string(),
        user_id: t.user_id.to_string(),
        amount_paise: t.amount_paise.to_string(),
        transaction_date: t.transaction_date,
        r#type: t.r#type,
    }
}

fn txs_response_to_vec(resp: TransactionsResponse) -> Vec<Transaction> {
    resp.items.into_iter().map(tx_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_transaction(
    input: NewTransaction,
) -> Result<Vec<Transaction>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_transaction(CreateTransactionRequest {
            user_id: parse_i64(&input.user_id, "user_id")?,
            amount_paise: parse_i64(&input.amount_paise, "amount_paise")?,
            r#type: input.r#type,
        })
        .await?;
    Ok(txs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_transaction(
    input: SearchTransactionInput,
) -> Result<Vec<Transaction>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_transaction(SearchTransactionRequest {
            transaction_id: parse_i64(&input.transaction_id, "transaction_id")?,
        })
        .await?;
    Ok(txs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_transaction(
    input: TransactionMutation,
) -> Result<Vec<Transaction>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_transaction(UpdateTransactionRequest {
            transaction_id: parse_i64(&input.transaction_id, "transaction_id")?,
            user_id: input
                .user_id
                .as_deref()
                .map(|s| parse_i64(s, "user_id"))
                .transpose()?,
            amount_paise: input
                .amount_paise
                .as_deref()
                .map(|s| parse_i64(s, "amount_paise"))
                .transpose()?,
            r#type: input.r#type,
        })
        .await?;
    Ok(txs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_transaction(
    input: DeleteTransactionInput,
) -> Result<Vec<Transaction>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_transaction(DeleteTransactionRequest {
            transaction_id: parse_i64(&input.transaction_id, "transaction_id")?,
        })
        .await?;
    Ok(txs_response_to_vec(resp.into_inner()))
}
