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
            let mut items: Vec<ProductImageResponse> = Vec::new();
            for model in models {
                let mut ordered: Vec<(i32, String)> = model
                    .urls
                    .as_object()
                    .map(|m| {
                        m.iter()
                            .filter_map(|(k, v)| {
                                let idx = k.parse::<i32>().ok()?;
                                let url = v.as_str()?.to_string();
                                Some((idx, url))
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                ordered.sort_by_key(|(idx, _)| *idx);

                for (idx, url) in ordered {
                    items.push(ProductImageResponse {
                        image_id: model.image_id,
                        product_id: model.product_id,
                        image_base64: String::new(),
                        alt_text: None,
                        url: Some(url),
                        cdn_path: None,
                        thumbnail_url: None,
                    });
                    // keep compiler happy about idx until dedicated field exists
                    let _ = idx;
                }
            }

            let response = ProductImagesResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
