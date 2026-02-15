use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_color_mapping;
use proto::proto::core::{
    DeleteProductColorMappingRequest, ProductColorMappingResponse,
    ProductColorMappingsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_product_color_mapping(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductColorMappingRequest>,
) -> Result<Response<ProductColorMappingsResponse>, Status> {
    let req = request.into_inner();

    let found = product_color_mapping::Entity::find()
        .filter(product_color_mapping::Column::ProductId.eq(req.product_id))
        .filter(product_color_mapping::Column::ColorId.eq(req.color_id))
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_color_mapping::Entity::delete_many()
                .filter(product_color_mapping::Column::ProductId.eq(req.product_id))
                .filter(product_color_mapping::Column::ColorId.eq(req.color_id))
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductColorMappingsResponse {
                    items: vec![ProductColorMappingResponse {
                        product_id: model.product_id,
                        color_id: model.color_id,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found("ProductColorMapping not found")),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
