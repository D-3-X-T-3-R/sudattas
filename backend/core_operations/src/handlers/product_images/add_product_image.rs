use crate::handlers::db_errors::map_db_error_to_status;
use base64;
use core_db_entities::entity::product_images;
use proto::proto::core::{AddProductImageRequest, ProductImageResponse, ProductImagesResponse};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use std::fs;
use tonic::{Request, Response, Status};

pub async fn add_product_image(
    txn: &DatabaseTransaction,
    request: Request<AddProductImageRequest>,
) -> Result<Response<ProductImagesResponse>, Status> {
    let req = request.into_inner();

    let mut images_response = Vec::new();
    for images in req.product_images.into_iter() {
        let product_image = product_images::ActiveModel {
            image_id: ActiveValue::NotSet,
            product_id: ActiveValue::Set(req.product_id),
            image_base64: ActiveValue::Set(Some(images.image_base64)),
            alt_text: ActiveValue::Set(images.alt_text),
            url: ActiveValue::NotSet,
            cdn_path: ActiveValue::NotSet,
            thumbnail_url: ActiveValue::NotSet,
            file_size_bytes: ActiveValue::NotSet,
            display_order: ActiveValue::NotSet,
            created_at: ActiveValue::NotSet,
        }
        .insert(txn)
        .await;

        match product_image {
            Ok(product_image) => {
                images_response.push(ProductImageResponse {
                    image_id: product_image.image_id,
                    product_id: product_image.product_id,
                    image_base64: product_image.image_base64.unwrap_or_default(),
                    alt_text: product_image.alt_text,
                });
            }
            Err(e) => return Err(map_db_error_to_status(e)),
        }
    }
    // match product_image.insert(txn).await {
    //     Ok(model) => {
    //         let response = ProductImagesResponse {
    //             items: vec![ProductImageResponse {
    //                 image_id: model.image_id,
    //                 product_id: model.product_id,
    //                 image_base64: model.image_base64,
    //                 alt_text: model.alt_text.unwrap(),
    //             }],
    //         };
    //         Ok(Response::new(response))
    //     }
    //     Err(e) => Err(map_db_error_to_status(e)),
    // }
    Ok(Response::new(ProductImagesResponse {
        items: images_response,
    }))
}
