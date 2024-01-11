use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::categories;
use proto::proto::core::{CategoriesResponse, CategoryResponse, DeleteCategoryRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_category(
    txn: &DatabaseTransaction,
    request: Request<DeleteCategoryRequest>,
) -> Result<Response<CategoriesResponse>, Status> {
    let req = request.into_inner();

    let category = categories::Entity::find_by_id(req.category_id)
        .one(txn)
        .await;

    match category {
        Ok(Some(model)) => {
            match categories::Entity::delete_many()
                .filter(categories::Column::CategoryId.eq(req.category_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = CategoriesResponse {
                            items: vec![CategoryResponse {
                                name: model.name,
                                category_id: model.category_id,
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "Category with ID {} not found.",
                            req.category_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Category item with ID {} not found.",
            req.category_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
