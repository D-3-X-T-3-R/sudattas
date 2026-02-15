use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_category_mapping;
use proto::proto::core::{
    DeleteProductCategoryMappingRequest, ProductCategoryMappingResponse,
    ProductCategoryMappingsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_product_category_mapping(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductCategoryMappingRequest>,
) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
    let req = request.into_inner();

    let found = product_category_mapping::Entity::find()
        .filter(product_category_mapping::Column::ProductId.eq(req.product_id))
        .filter(product_category_mapping::Column::CategoryId.eq(req.category_id))
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_category_mapping::Entity::delete_many()
                .filter(product_category_mapping::Column::ProductId.eq(req.product_id))
                .filter(product_category_mapping::Column::CategoryId.eq(req.category_id))
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductCategoryMappingsResponse {
                    items: vec![ProductCategoryMappingResponse {
                        product_id: model.product_id,
                        category_id: model.category_id,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found("ProductCategoryMapping not found")),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
