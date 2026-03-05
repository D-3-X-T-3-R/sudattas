use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_attributes;
use proto::proto::core::{
    ProductAttributeResponse, ProductAttributesResponse, SearchProductAttributeRequest,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_product_attribute(
    txn: &DatabaseTransaction,
    request: Request<SearchProductAttributeRequest>,
) -> Result<Response<ProductAttributesResponse>, Status> {
    let req = request.into_inner();

    let mut query = product_attributes::Entity::find();
    if let Some(attribute_id) = req.attribute_id {
        if attribute_id != 0 {
            query = query.filter(product_attributes::Column::AttributeId.eq(attribute_id));
        }
    }
    if let Some(ref name) = req.attribute_name {
        query = query.filter(product_attributes::Column::AttributeName.eq(name.as_str()));
    }
    if let Some(ref value) = req.attribute_value {
        query = query.filter(product_attributes::Column::AttributeValue.eq(value.as_str()));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ProductAttributeResponse {
                    attribute_id: m.attribute_id,
                    attribute_name: m.attribute_name,
                    attribute_value: m.attribute_value,
                })
                .collect();
            Ok(Response::new(ProductAttributesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
