use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::decimal_to_paise;
use core_db_entities::entity::product_variants;
use proto::proto::core::{
    DeleteProductVariantRequest, ProductVariantResponse, ProductVariantsResponse,
};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_product_variant(
    txn: &DatabaseTransaction,
    request: Request<DeleteProductVariantRequest>,
) -> Result<Response<ProductVariantsResponse>, Status> {
    let req = request.into_inner();

    let found = product_variants::Entity::find_by_id(req.variant_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match product_variants::Entity::delete_by_id(req.variant_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(ProductVariantsResponse {
                    items: vec![ProductVariantResponse {
                        variant_id: model.variant_id,
                        product_id: model.product_id,
                        size_id: model.size_id,
                        color_id: model.color_id,
                        additional_price_paise: model
                            .additional_price
                            .as_ref()
                            .map(decimal_to_paise),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "ProductVariant with ID {} not found",
            req.variant_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
