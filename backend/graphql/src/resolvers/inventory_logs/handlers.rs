use proto::proto::core::{
    CreateInventoryLogRequest, DeleteInventoryLogRequest, InventoryLogResponse,
    InventoryLogsResponse, SearchInventoryLogRequest, UpdateInventoryLogRequest,
};
use tracing::instrument;

use super::schema::{
    DeleteInventoryLogInput, InventoryLog, InventoryLogMutation, NewInventoryLog,
    SearchInventoryLogInput,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_option_i64},
};

fn log_response_to_gql(l: InventoryLogResponse) -> InventoryLog {
    InventoryLog {
        log_id: l.log_id.to_string(),
        variant_id: l.variant_id.to_string(),
        change_quantity: l.change_quantity.to_string(),
        log_time: l.log_time,
        reason: l.reason,
    }
}

fn logs_response_to_vec(resp: InventoryLogsResponse) -> Vec<InventoryLog> {
    resp.items.into_iter().map(log_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_inventory_log(
    input: NewInventoryLog,
) -> Result<Vec<InventoryLog>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_inventory_log(CreateInventoryLogRequest {
            variant_id: parse_i64(&input.variant_id, "variant_id")?,
            change_quantity: parse_i64(&input.change_quantity, "change_quantity")?,
            reason: input.reason,
            actor_id: input.actor_id,
            quantity_before: to_option_i64(input.quantity_before),
            quantity_after: to_option_i64(input.quantity_after),
        })
        .await?;
    Ok(logs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_inventory_log(
    input: SearchInventoryLogInput,
) -> Result<Vec<InventoryLog>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_inventory_log(SearchInventoryLogRequest {
            log_id: to_option_i64(input.log_id),
            variant_id: to_option_i64(input.variant_id),
        })
        .await?;
    Ok(logs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_inventory_log(
    input: InventoryLogMutation,
) -> Result<Vec<InventoryLog>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_inventory_log(UpdateInventoryLogRequest {
            log_id: parse_i64(&input.log_id, "log_id")?,
            variant_id: input
                .variant_id
                .as_deref()
                .map(|s| parse_i64(s, "variant_id"))
                .transpose()?,
            change_quantity: input
                .change_quantity
                .as_deref()
                .map(|s| parse_i64(s, "change_quantity"))
                .transpose()?,
            reason: input.reason,
        })
        .await?;
    Ok(logs_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_inventory_log(
    input: DeleteInventoryLogInput,
) -> Result<Vec<InventoryLog>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_inventory_log(DeleteInventoryLogRequest {
            log_id: parse_i64(&input.log_id, "log_id")?,
        })
        .await?;
    Ok(logs_response_to_vec(resp.into_inner()))
}
