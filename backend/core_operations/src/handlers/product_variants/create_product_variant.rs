use crate::handlers::db_errors::map_db_error_to_status;
use crate::money::{decimal_to_paise, paise_to_decimal};
use core_db_entities::entity::product_variants;
use proto::proto::core::{
    CreateProductVariantRequest, ProductVariantResponse, ProductVariantsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_product_variant(
    txn: &DatabaseTransaction,
    request: Request<CreateProductVariantRequest>,
) -> Result<Response<ProductVariantsResponse>, Status> {
    let req = request.into_inner();
    let additional_price = req.additional_price_paise.map(paise_to_decimal);
    let model = product_variants::ActiveModel {
        variant_id: ActiveValue::NotSet,
        product_id: ActiveValue::Set(req.product_id),
        size_id: ActiveValue::Set(req.size_id),
        color_id: ActiveValue::Set(req.color_id),
        additional_price: ActiveValue::Set(additional_price),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ProductVariantsResponse {
            items: vec![ProductVariantResponse {
                variant_id: inserted.variant_id,
                product_id: inserted.product_id,
                size_id: inserted.size_id,
                color_id: inserted.color_id,
                additional_price_paise: inserted.additional_price.as_ref().map(decimal_to_paise),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
