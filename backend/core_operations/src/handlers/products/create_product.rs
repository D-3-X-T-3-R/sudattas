use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{CreateProductRequest, ProductResponse, ProductsResponse};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, QueryResult};
use tonic::{Request, Response, Status};

pub async fn create_product(
    db: &DatabaseConnection,
    request: Request<CreateProductRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();
    let product = products::ActiveModel {
        product_id: ActiveValue::NotSet,
        name: ActiveValue::Set(req.name),
        description: ActiveValue::Set(req.description),
        price: ActiveValue::Set(Decimal::from_f64(req.price).unwrap()),
        stock_quantity: ActiveValue::Set(req.stock_quantity),
        category_id: ActiveValue::Set(req.category_id),
    };
    match product.insert(db).await {
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
