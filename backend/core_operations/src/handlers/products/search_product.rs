use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::products;
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
        .apply_if(req.starting_price_paise, |query, v| {
            query.filter(products::Column::Price.gte(paise_to_decimal(v)))
        })
        .apply_if(req.ending_price_paise, |query, v| {
            query.filter(products::Column::Price.lte(paise_to_decimal(v)))
        })
        .apply_if(req.limit, |query, v| query.limit(v as u64))
        .apply_if(req.offset, |query, v| query.offset(v as u64))
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| {
                    let price_paise = model
                        .price_paise
                        .map(i64::from)
                        .unwrap_or_else(|| decimal_to_paise(&model.price));
                    ProductResponse {
                        name: model.name,
                        product_id: model.product_id,
                        description: model.description,
                        price_paise,
                        stock_quantity: model.stock_quantity,
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
