use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{ProductResponse, ProductsResponse, SearchProductRequest};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_product(
    db: &DatabaseConnection,
    request: Request<SearchProductRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let req = request.into_inner();

    match products::Entity::find()
        .apply_if(req.product_id, |query, v| {
            query.filter(products::Column::ProductId.eq(v))
        })
        .apply_if(req.name, |query, v| {
            query.filter(products::Column::Name.contains(v))
        })
        .apply_if(req.description, |query, v| {
            query.filter(products::Column::Description.contains(v))
        })
        .apply_if(req.stock_quantity, |query, v| {
            query.filter(products::Column::StockQuantity.gte(v))
        })
        .apply_if(req.category_id, |query, v| {
            query.filter(products::Column::CategoryId.eq(v))
        })
        .apply_if(req.starting_price, |query, v| {
            query.filter(products::Column::Price.gte(v))
        })
        .apply_if(req.ending_price, |query, v| {
            query.filter(products::Column::Price.lte(v))
        })
        .all(db)
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
