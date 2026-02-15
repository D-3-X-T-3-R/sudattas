use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_attribute_mapping;
use proto::proto::core::{
    DeleteProductAttributeMappingRequest, ProductAttributeMappingResponse,
    ProductAttributeMappingsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_product_attribute_mapping(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductAttributeMappingRequest>,
) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
    let req = request.into_inner();

    let found = product_attribute_mapping::Entity::find()
        .filter(product_attribute_mapping::Column::ProductId.eq(req.product_id))
        .filter(product_attribute_mapping::Column::AttributeId.eq(req.attribute_id))
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_attribute_mapping::Entity::delete_many()
                .filter(product_attribute_mapping::Column::ProductId.eq(req.product_id))
                .filter(product_attribute_mapping::Column::AttributeId.eq(req.attribute_id))
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductAttributeMappingsResponse {
                    items: vec![ProductAttributeMappingResponse {
                        product_id: model.product_id,
                        attribute_id: model.attribute_id,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found("ProductAttributeMapping not found")),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
