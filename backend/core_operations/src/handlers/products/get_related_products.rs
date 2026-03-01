//! P2 Recommendations: return manually linked related products for a product.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::decimal_to_paise;
use core_db_entities::entity::{product_related, products};
use proto::proto::core::{GetRelatedProductsRequest, ProductResponse, ProductsResponse};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

const DEFAULT_LIMIT: u64 = 20;

pub async fn get_related_products(
    txn: &DatabaseTransaction,
    request: Request<GetRelatedProductsRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();
    let limit = req
        .limit
        .map(|l| l as u64)
        .unwrap_or(DEFAULT_LIMIT)
        .min(100);

    let related_rows = product_related::Entity::find()
        .filter(product_related::Column::ProductId.eq(req.product_id))
        .order_by_asc(product_related::Column::DisplayOrder)
        .limit(limit)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let related_ids: Vec<i64> = related_rows
        .into_iter()
        .map(|r| r.related_product_id)
        .collect();

    if related_ids.is_empty() {
        return Ok(Response::new(ProductsResponse { items: vec![] }));
    }

    let product_models = products::Entity::find()
        .filter(products::Column::ProductId.is_in(related_ids.clone()))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let by_id: HashMap<i64, products::Model> = product_models
        .into_iter()
        .map(|p| (p.product_id, p))
        .collect();

    let items: Vec<ProductResponse> = related_ids
        .into_iter()
        .filter_map(|id| by_id.get(&id))
        .map(|model| {
            let price_paise = model
                .price_paise
                .map(i64::from)
                .unwrap_or_else(|| decimal_to_paise(&model.price));
            ProductResponse {
                name: model.name.clone(),
                product_id: model.product_id,
                description: model.description.clone(),
                price_paise,
                stock_quantity: model.stock_quantity,
                category_id: model.category_id,
            }
        })
        .collect();

    Ok(Response::new(ProductsResponse { items }))
}
