use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_color_mapping;
use proto::proto::core::{
    ProductColorMappingResponse, ProductColorMappingsResponse, SearchProductColorMappingRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_product_color_mapping(
    txn: &DatabaseTransaction,
    request: Request<SearchProductColorMappingRequest>,
) -> Result<Response<ProductColorMappingsResponse>, Status> {
    let req = request.into_inner();

    match product_color_mapping::Entity::find()
        .apply_if(Some(req.product_id), |query, v| {
            query.filter(product_color_mapping::Column::ProductId.eq(v))
        })
        .apply_if(Some(req.color_id), |query, v| {
            query.filter(product_color_mapping::Column::ColorId.eq(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductColorMappingResponse {
                    product_id: m.product_id,
                    color_id: m.color_id,
                })
                .collect();
            Ok(Response::new(ProductColorMappingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
