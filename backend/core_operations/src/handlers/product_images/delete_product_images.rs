use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_images;
use proto::proto::core::{DeleteProductImageRequest, ProductImageResponse, ProductImagesResponse};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_product_image(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductImageRequest>,
) -> Result<Response<ProductImagesResponse>, Status> {
    let req = request.into_inner();

    let product_image = product_images::Entity::find_by_id(req.image_id)
        .one(txn)
        .await;

    match product_image {
        Ok(Some(model)) => {
            match product_images::Entity::delete_many()
                .filter(product_images::Column::ImageId.eq(req.image_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = ProductImagesResponse {
                            items: vec![ProductImageResponse {
                                image_id: model.image_id,
                                product_id: model.product_id,
                                image_base64: model.image_base64,
                                alt_text: model.alt_text,
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "ProductImage with ID {} not found.",
                            req.image_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ProductImage with ID {} not found.",
            req.image_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
