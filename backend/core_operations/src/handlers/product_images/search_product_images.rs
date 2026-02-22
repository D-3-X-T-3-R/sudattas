use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_images;
use proto::proto::core::{ProductImageResponse, ProductImagesResponse, SearchProductImageRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_product_image(
    txn: &DatabaseTransaction,
    request: Request<SearchProductImageRequest>,
) -> Result<Response<ProductImagesResponse>, Status> {
    let req = request.into_inner();

    match product_images::Entity::find()
        .apply_if(req.image_id, |query, v| {
            query.filter(product_images::Column::ImageId.eq(v))
        })
        .apply_if(req.product_id, |query, v| {
            query.filter(product_images::Column::ProductId.eq(v))
        })
        .apply_if(req.alt_text, |query, v| {
            query.filter(product_images::Column::AltText.starts_with(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| ProductImageResponse {
                    image_id: model.image_id,
                    product_id: model.product_id,
                    image_base64: model.image_base64.unwrap_or_default(),
                    alt_text: model.alt_text,
                })
                .collect();

            let response = ProductImagesResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
