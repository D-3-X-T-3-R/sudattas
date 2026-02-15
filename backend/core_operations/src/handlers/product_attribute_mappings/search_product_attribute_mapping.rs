use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_attribute_mapping;
use proto::proto::core::{
    ProductAttributeMappingResponse, ProductAttributeMappingsResponse,
    SearchProductAttributeMappingRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_product_attribute_mapping(
    txn: &DatabaseTransaction,
    request: Request<SearchProductAttributeMappingRequest>,
) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
    let req = request.into_inner();

    match product_attribute_mapping::Entity::find()
        .filter(product_attribute_mapping::Column::ProductId.eq(req.product_id))
        .filter(product_attribute_mapping::Column::AttributeId.eq(req.attribute_id))
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductAttributeMappingResponse {
                    product_id: m.product_id,
                    attribute_id: m.attribute_id,
                })
                .collect();
            Ok(Response::new(ProductAttributeMappingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
