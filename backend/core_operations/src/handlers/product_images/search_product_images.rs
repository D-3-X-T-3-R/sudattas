use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_images;
use proto::proto::core::{ProductImageResponse, ProductImagesResponse, SearchProductImageRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder};
use tonic::{Request, Response, Status};

pub async fn search_product_image(
    txn: &DatabaseTransaction,
    request: Request<SearchProductImageRequest>,
) -> Result<Response<ProductImagesResponse>, Status> {
    let req = request.into_inner();

    let mut query =
        product_images::Entity::find().order_by_asc(product_images::Column::DisplayOrder);

    if let Some(id) = req.image_id {
        query = query.filter(product_images::Column::ImageId.eq(id));
    }
    if let Some(pid) = req.product_id {
        query = query.filter(product_images::Column::ProductId.eq(pid));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items: Vec<ProductImageResponse> = models
                .into_iter()
                .map(|m| ProductImageResponse {
                    image_id: m.image_id,
                    product_id: m.product_id,
                    image_base64: String::new(),
                    alt_text: None,
                    url: Some(m.url.clone()),
                    cdn_path: None,
                    thumbnail_url: Some(m.url),
                })
                .collect();
            Ok(Response::new(ProductImagesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
