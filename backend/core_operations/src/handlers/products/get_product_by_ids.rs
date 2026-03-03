use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::decimal_to_paise;
use core_db_entities::entity::products;
use proto::proto::core::{GetProductsByIdRequest, ProductResponse, ProductsResponse};
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
                .map(|model| {
                    let price_paise = model.price_paise as i64;
                    ProductResponse {
                        name: model.name,
                        product_id: model.product_id,
                        description: model.description,
                        price_paise,
                        category_id: model.category_id,
                    }
                })
                .collect();

            let response = ProductsResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
