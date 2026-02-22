use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_category_mapping;
use proto::proto::core::{
    ProductCategoryMappingResponse, ProductCategoryMappingsResponse,
    SearchProductCategoryMappingRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_product_category_mapping(
    txn: &DatabaseTransaction,
    request: Request<SearchProductCategoryMappingRequest>,
) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
    let req = request.into_inner();

    match product_category_mapping::Entity::find()
        .filter(product_category_mapping::Column::ProductId.eq(req.product_id))
        .filter(product_category_mapping::Column::CategoryId.eq(req.category_id))
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductCategoryMappingResponse {
                    product_id: m.product_id,
                    category_id: m.category_id,
                })
                .collect();
            Ok(Response::new(ProductCategoryMappingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
