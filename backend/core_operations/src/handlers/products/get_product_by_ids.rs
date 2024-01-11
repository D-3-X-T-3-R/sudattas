use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{GetProductsByIdRequest, ProductResponse, ProductsResponse};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn get_products_by_id(
    txn: &DatabaseTransaction,
    request: Request<GetProductsByIdRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();

    match products::Entity::find()
        .filter(products::Column::ProductId.is_in(req.product_ids))
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| ProductResponse {
                    name: model.name,
                    product_id: model.product_id,
                    description: model.description,
                    price: Decimal::to_f64(&model.price).unwrap(),
                    stock_quantity: model.stock_quantity,
                    category_id: model.category_id,
                })
                .collect();

            let response = ProductsResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
