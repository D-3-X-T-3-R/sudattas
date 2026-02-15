use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_size_mapping;
use proto::proto::core::{
    DeleteProductSizeMappingRequest, ProductSizeMappingResponse,
    ProductSizeMappingsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_product_size_mapping(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductSizeMappingRequest>,
) -> Result<Response<ProductSizeMappingsResponse>, Status> {
    let req = request.into_inner();

    let found = product_size_mapping::Entity::find()
        .filter(product_size_mapping::Column::ProductId.eq(req.product_id))
        .filter(product_size_mapping::Column::SizeId.eq(req.size_id))
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_size_mapping::Entity::delete_many()
                .filter(product_size_mapping::Column::ProductId.eq(req.product_id))
                .filter(product_size_mapping::Column::SizeId.eq(req.size_id))
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductSizeMappingsResponse {
                    items: vec![ProductSizeMappingResponse {
                        product_id: model.product_id,
                        size_id: model.size_id,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found("ProductSizeMapping not found")),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
