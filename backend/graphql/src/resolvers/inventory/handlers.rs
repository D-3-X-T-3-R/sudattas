use proto::proto::core::{
    CreateInventoryItemRequest, DeleteInventoryItemRequest, InventoryItemResponse,
    SearchInventoryItemRequest, UpdateInventoryItemRequest,
};
use tracing::instrument;

use super::schema::{InventoryItem, InventoryItemMutation, NewInventoryItem, SearchInventoryItem};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn inventory_response_to_gql(i: InventoryItemResponse) -> InventoryItem {
    InventoryItem {
        inventory_id: i.inventory_id.to_string(),
        product_id: i.product_id.to_string(),
        quantity_available: i.quantity_available.to_string(),
        reorder_level: i.reorder_level.to_string(),
        supplier_id: i.supplier_id.to_string(),
    }
}

#[instrument]
pub(crate) async fn search_inventory_item(
    input: SearchInventoryItem,
) -> Result<Vec<InventoryItem>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .search_inventory_item(SearchInventoryItemRequest {
            inventory_id: input
                .inventory_id
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            product_id: input
                .product_id
                .as_deref()
                .map(|s| parse_i64(s, "product id"))
                .transpose()?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(inventory_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn create_inventory_item(
    input: NewInventoryItem,
) -> Result<Vec<InventoryItem>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_inventory_item(CreateInventoryItemRequest {
            product_id: parse_i64(&input.product_id, "product id")?,
            quantity_available: parse_i64(&input.quantity_available, "quantity_available")?,
            reorder_level: input
                .reorder_level
                .as_deref()
                .map(|s| parse_i64(s, "reorder_level"))
                .transpose()?
                .unwrap_or(0),
            supplier_id: input
                .supplier_id
                .as_deref()
                .map(|s| parse_i64(s, "supplier id"))
                .transpose()?
                .unwrap_or(0),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(inventory_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_inventory_item(
    input: InventoryItemMutation,
) -> Result<Vec<InventoryItem>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_inventory_item(UpdateInventoryItemRequest {
            inventory_id: parse_i64(&input.inventory_id, "inventory id")?,
            product_id: input
                .product_id
                .as_deref()
                .map(|s| parse_i64(s, "product id"))
                .transpose()?,
            quantity_available: input
                .quantity_available
                .as_deref()
                .map(|s| parse_i64(s, "quantity_available"))
                .transpose()?,
            reorder_level: input
                .reorder_level
                .as_deref()
                .map(|s| parse_i64(s, "reorder_level"))
                .transpose()?,
            supplier_id: input
                .supplier_id
                .as_deref()
                .map(|s| parse_i64(s, "supplier id"))
                .transpose()?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(inventory_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_inventory_item(
    inventory_id: String,
) -> Result<Vec<InventoryItem>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .delete_inventory_item(DeleteInventoryItemRequest {
            inventory_id: parse_i64(&inventory_id, "inventory id")?,
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(inventory_response_to_gql)
        .collect())
}
