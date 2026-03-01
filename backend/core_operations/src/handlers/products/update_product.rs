use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::products;
use proto::proto::core::{ProductResponse, ProductsResponse, UpdateProductRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_product(
    txn: &DatabaseTransaction,
    request: Request<UpdateProductRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();

    let products = products::ActiveModel {
        name: ActiveValue::Set(req.name),
        description: ActiveValue::Set(req.description),
        price: ActiveValue::Set(paise_to_decimal(req.price_paise)),
        product_id: ActiveValue::Set(req.product_id),
        stock_quantity: ActiveValue::Set(req.stock_quantity),
        category_id: ActiveValue::Set(req.category_id),
        sku: ActiveValue::NotSet,
        slug: ActiveValue::NotSet,
        price_paise: ActiveValue::Set(Some(req.price_paise as i32)),
        fabric: ActiveValue::NotSet,
        weave: ActiveValue::NotSet,
        occasion: ActiveValue::NotSet,
        length_meters: ActiveValue::NotSet,
        has_blouse_piece: ActiveValue::NotSet,
        care_instructions: ActiveValue::NotSet,
        product_status_id: ActiveValue::NotSet,
        created_at: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    };
    match products.update(txn).await {
        Ok(model) => {
            let price_paise = model
                .price_paise
                .map(i64::from)
                .unwrap_or_else(|| decimal_to_paise(&model.price));
            let response = ProductsResponse {
                items: vec![ProductResponse {
                    name: model.name,
                    product_id: model.product_id,
                    description: model.description,
                    price_paise,
                    stock_quantity: model.stock_quantity,
                    category_id: model.category_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
