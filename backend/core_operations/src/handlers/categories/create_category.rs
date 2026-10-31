use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::product_categories;
use proto::proto::core::{CategoriesResponse, CategoryResponse, CreateCategoryRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_category(
    txn: &DatabaseTransaction,
    request: Request<CreateCategoryRequest>,
) -> Result<Response<CategoriesResponse>, Status> {
    let req = request.into_inner();
    let product = product_categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(req.name.clone()),
        exchange_eligible: ActiveValue::Set(i8::from(req.exchange_eligible.unwrap_or(false))),
    };

    match product.insert(txn).await {
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
