use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{DeleteProductRequest, ProductResponse, ProductsResponse};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_product(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();

    let product = products::Entity::find_by_id(req.product_id).one(txn).await;

    match product {
        Ok(Some(model)) => {
            match products::Entity::delete_many()
                .filter(products::Column::ProductId.eq(req.product_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
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
                    } else {
                        Err(Status::not_found(format!(
                            "Product with ID {} not found.",
                            req.product_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Product with ID {} not found.",
            req.product_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
