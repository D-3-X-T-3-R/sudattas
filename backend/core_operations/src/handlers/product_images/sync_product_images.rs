//! Sync product images: update display_order for kept rows, bulk insert new, delete removed.

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_images;
use proto::proto::core::{ProductImageResponse, ProductImagesResponse, SyncProductImagesRequest};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use tonic::{Request, Response, Status};

fn url_from_key(key: &str) -> String {
    let base = std::env::var("R2_PUBLIC_URL")
        .unwrap_or_else(|_| "https://images.example.com".to_string())
        .trim_end_matches('/')
        .to_string();
    format!("{}/{}", base, key)
}

pub async fn sync_product_images(
    txn: &DatabaseTransaction,
    request: Request<SyncProductImagesRequest>,
) -> Result<Response<ProductImagesResponse>, Status> {
    let req = request.into_inner();
    if req.items.is_empty() {
        product_images::Entity::delete_many()
            .filter(product_images::Column::ProductId.eq(req.product_id))
            .exec(txn)
            .await
            .map_err(map_db_error_to_status)?;
        return Ok(Response::new(ProductImagesResponse { items: vec![] }));
    }

    let mut kept_ids: Vec<i64> = Vec::new();
    for (index, item) in req.items.iter().enumerate() {
        let display_order = index as i32;
        if let Some(image_id) = item.image_id {
            kept_ids.push(image_id);
            if let Some(row) = product_images::Entity::find_by_id(image_id)
                .one(txn)
                .await
                .map_err(map_db_error_to_status)?
            {
                if row.product_id == req.product_id {
                    let mut am: product_images::ActiveModel = row.into();
                    am.display_order = Set(display_order);
                    am.update(txn).await.map_err(map_db_error_to_status)?;
                }
            }
        } else {
            let url = item
                .url
                .clone()
                .unwrap_or_else(|| url_from_key(item.key.as_deref().unwrap_or("")));
            let inserted = product_images::ActiveModel {
                image_id: ActiveValue::NotSet,
                product_id: ActiveValue::Set(req.product_id),
                display_order: ActiveValue::Set(display_order),
                url: ActiveValue::Set(url),
                created_at: ActiveValue::NotSet,
            }
            .insert(txn)
            .await
            .map_err(map_db_error_to_status)?;
            kept_ids.push(inserted.image_id);
        }
    }

    if !kept_ids.is_empty() {
        product_images::Entity::delete_many()
            .filter(product_images::Column::ProductId.eq(req.product_id))
            .filter(product_images::Column::ImageId.is_not_in(kept_ids))
            .exec(txn)
            .await
            .map_err(map_db_error_to_status)?;
    }

    let models = product_images::Entity::find()
        .filter(product_images::Column::ProductId.eq(req.product_id))
        .order_by_asc(product_images::Column::DisplayOrder)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

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
