use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::categories;
use proto::proto::core::{CategoriesResponse, CategoryResponse, CreateCategoryRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use tonic::{Request, Response, Status};

pub async fn create_category(
    db: &DatabaseConnection,
    request: Request<CreateCategoryRequest>,
) -> Result<Response<CategoriesResponse>, Status> {
    let req = request.into_inner();
    let product = categories::ActiveModel {
        category_id: ActiveValue::NotSet,
        name: ActiveValue::Set(req.name),
    };

    match product.insert(db).await {
        Ok(model) => {
            let response = CategoriesResponse {
                items: vec![CategoryResponse {
                    name: model.name,
                    category_id: model.category_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
