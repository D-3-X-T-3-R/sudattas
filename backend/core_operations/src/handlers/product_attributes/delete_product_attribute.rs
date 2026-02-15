use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_attributes;
use proto::proto::core::{
    DeleteProductAttributeRequest, ProductAttributeResponse, ProductAttributesResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_product_attribute(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductAttributeRequest>,
) -> Result<Response<ProductAttributesResponse>, Status> {
    let req = request.into_inner();

    let found = product_attributes::Entity::find_by_id(req.attribute_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_attributes::Entity::delete_by_id(req.attribute_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductAttributesResponse {
                    items: vec![ProductAttributeResponse {
                        attribute_id: model.attribute_id,
                        product_id: model.product_id.unwrap_or(0),
                        attribute_name: model.attribute_name.unwrap_or_default(),
                        attribute_value: model.attribute_value.unwrap_or_default(),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ProductAttribute with ID {} not found",
            req.attribute_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
