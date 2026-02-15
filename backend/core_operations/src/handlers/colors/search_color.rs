use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::colors;
use proto::proto::core::{ColorResponse, ColorsResponse, SearchColorRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_color(
    txn: &DatabaseTransaction,
    request: Request<SearchColorRequest>,
) -> Result<Response<ColorsResponse>, Status> {
    let req = request.into_inner();

    let mut query = colors::Entity::find();
    if req.color_id != 0 {
        query = query.filter(colors::Column::ColorId.eq(req.color_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| ColorResponse {
                    color_id: m.color_id,
                    color_name: m.color_name,
                })
                .collect();
            Ok(Response::new(ColorsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
