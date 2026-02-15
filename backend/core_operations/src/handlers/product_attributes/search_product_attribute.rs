use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_attributes;
use proto::proto::core::{
    ProductAttributeResponse, ProductAttributesResponse, SearchProductAttributeRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_product_attribute(
    txn: &DatabaseTransaction,
    request: Request<SearchProductAttributeRequest>,
) -> Result<Response<ProductAttributesResponse>, Status> {
    let req = request.into_inner();

    let mut query = product_attributes::Entity::find();
    if req.attribute_id != 0 {
        query = query.filter(product_attributes::Column::AttributeId.eq(req.attribute_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductAttributeResponse {
                    attribute_id: m.attribute_id,
                    product_id: m.product_id.unwrap_or(0),
                    attribute_name: m.attribute_name.unwrap_or_default(),
                    attribute_value: m.attribute_value.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(ProductAttributesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
