use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{ProductResponse, ProductsResponse, UpdateProductRequest};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
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
        price: ActiveValue::Set(Decimal::from_f64(req.price).unwrap()),
        product_id: ActiveValue::Set(req.product_id),
        stock_quantity: ActiveValue::Set(req.stock_quantity),
        category_id: ActiveValue::Set(req.category_id),
    };
    match products.update(txn).await {
        Ok(model) => {
            let response = ProductsResponse {
                items: vec![ProductResponse {
                    name: model.name,
                    product_id: model.product_id,
                    description: model.description,
                    price: Decimal::to_f64(&model.price).unwrap(),
                    stock_quantity: model.stock_quantity,
                    category_id: model.category_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
