use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::decimal_to_paise;
use core_db_entities::entity::product_variants;
use proto::proto::core::{
    ProductVariantResponse, ProductVariantsResponse, SearchProductVariantRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_product_variant(
    txn: &DatabaseTransaction,
    request: Request<SearchProductVariantRequest>,
) -> Result<Response<ProductVariantsResponse>, Status> {
    let req = request.into_inner();

    let mut query = product_variants::Entity::find();
    if req.variant_id != 0 {
        query = query.filter(product_variants::Column::VariantId.eq(req.variant_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductVariantResponse {
                    variant_id: m.variant_id,
                    product_id: m.product_id,
                    size_id: m.size_id,
                    color_id: m.color_id,
                    additional_price_paise: m.additional_price.as_ref().map(decimal_to_paise),
                })
                .collect();
            Ok(Response::new(ProductVariantsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
