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
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| {
                    let url = model
                        .urls
                        .get("1")
                        .or_else(|| model.urls.get("0"))
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    ProductImageResponse {
                        image_id: model.image_id,
                        product_id: model.product_id,
                        image_base64: String::new(),
                        alt_text: None,
                        url,
                        cdn_path: None,
                        thumbnail_url: None,
                    }
                })
                .collect();

            let response = ProductImagesResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
