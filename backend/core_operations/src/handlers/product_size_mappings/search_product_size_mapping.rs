use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_size_mapping;
use proto::proto::core::{
    ProductSizeMappingResponse, ProductSizeMappingsResponse,
    SearchProductSizeMappingRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_product_size_mapping(
    txn: &DatabaseTransaction,
    request: Request<SearchProductSizeMappingRequest>,
) -> Result<Response<ProductSizeMappingsResponse>, Status> {
    let req = request.into_inner();

    match product_size_mapping::Entity::find()
        .apply_if(Some(req.product_id), |query, v| {
            query.filter(product_size_mapping::Column::ProductId.eq(v))
        })
        .apply_if(Some(req.size_id), |query, v| {
            query.filter(product_size_mapping::Column::SizeId.eq(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductSizeMappingResponse {
                    product_id: m.product_id,
                    size_id: m.size_id,
                })
                .collect();
            Ok(Response::new(ProductSizeMappingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
