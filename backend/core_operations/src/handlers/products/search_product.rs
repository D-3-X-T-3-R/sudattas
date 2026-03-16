use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::{product_mood_mapping, products};
use proto::proto::core::{ProductResponse, ProductsResponse, SearchProductRequest};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, QueryTrait,
};
use tonic::{Request, Response, Status};

pub async fn search_product(
    txn: &DatabaseTransaction,
    request: Request<SearchProductRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();

    let mut query = products::Entity::find();

    if let Some(mood_id) = req.mood_id {
        if mood_id != 0 {
            let product_ids: Vec<i64> = product_mood_mapping::Entity::find()
                .filter(product_mood_mapping::Column::MoodId.eq(mood_id))
                .all(txn)
                .await
                .map_err(map_db_error_to_status)?
                .into_iter()
                .map(|m| m.product_id)
                .collect();
            if product_ids.is_empty() {
                return Ok(Response::new(ProductsResponse { items: vec![] }));
            }
            query = query.filter(products::Column::ProductId.is_in(product_ids));
        }
    }

    let query = query
        .apply_if(req.product_id, |q, v| {
            q.filter(products::Column::ProductId.eq(v))
        })
        .apply_if(req.name, |q, v| {
            q.filter(products::Column::Name.contains(v))
        })
        .apply_if(req.description, |q, v| {
            q.filter(products::Column::Description.contains(v))
        })
        .apply_if(req.category_id, |q, v| {
            q.filter(products::Column::CategoryId.eq(v))
        })
        .apply_if(req.starting_price_paise, |q, v| {
            q.filter(products::Column::PricePaise.gte(v as i32))
        })
        .apply_if(req.ending_price_paise, |q, v| {
            q.filter(products::Column::PricePaise.lte(v as i32))
        })
        .apply_if(req.fabric, |q, v| q.filter(products::Column::Fabric.eq(v)))
        .apply_if(req.weave, |q, v| q.filter(products::Column::Weave.eq(v)))
        .apply_if(req.occasion, |q, v| {
            q.filter(products::Column::Occasion.eq(v))
        })
        .apply_if(req.product_status_id, |q, v| {
            q.filter(products::Column::ProductStatusId.eq(v))
        })
        .apply_if(req.limit, |q, v| q.limit(v as u64))
        .apply_if(req.offset, |q, v| q.offset(v as u64));

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| {
                    let price_paise = model.price_paise as i64;
                    ProductResponse {
                        name: model.name,
                        product_id: model.product_id,
                        description: model.description,
                        price_paise,
                        category_id: model.category_id,
                        sku: model.sku,
                        slug: model.slug,
                        fabric: model.fabric,
                        weave: model.weave,
                        occasion: model.occasion,
                        has_blouse_piece: model.has_blouse_piece.map(|v| v != 0),
                        care_instructions: model.care_instructions,
                        product_status_id: model.product_status_id,
                    }
                })
                .collect();

            let response = ProductsResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
