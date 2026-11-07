use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_categories;
use proto::proto::core::{CategoriesResponse, CategoryResponse, UpdateCategoryRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_category(
    txn: &DatabaseTransaction,
    request: Request<UpdateCategoryRequest>,
) -> Result<Response<CategoriesResponse>, Status> {
    let req = request.into_inner();

    let categories = product_categories::ActiveModel {
        name: ActiveValue::Set(req.name),
        category_id: ActiveValue::Set(req.category_id),
        // Unset leaves the existing value untouched — SeaORM only issues SET for fields marked Set.
        exchange_eligible: match req.exchange_eligible {
            Some(v) => ActiveValue::Set(i8::from(v)),
            None => ActiveValue::NotSet,
        },
    };
    match categories.update(txn).await {
        Ok(model) => {
            let response = CategoriesResponse {
                items: vec![CategoryResponse {
                    name: model.name,
                    category_id: model.category_id,
                    exchange_eligible: model.exchange_eligible != 0,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
