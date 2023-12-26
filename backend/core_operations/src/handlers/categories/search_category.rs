use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::categories;
use proto::proto::core::{CategoriesResponse, CategoryResponse, SearchCategoryRequest};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_category(
    db: &DatabaseConnection,
    request: Request<SearchCategoryRequest>,
) -> Result<Response<CategoriesResponse>, Status> {
    let req = request.into_inner();

    match categories::Entity::find()
        .apply_if(req.category_id, |query, v| {
            query.filter(categories::Column::CategoryId.eq(v))
        })
        .apply_if(req.name, |query, v| {
            query.filter(categories::Column::Name.contains(v))
        })
        .all(db)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| CategoryResponse {
                    name: model.name,
                    category_id: model.category_id,
                })
                .collect();

            Ok(Response::new(CategoriesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
