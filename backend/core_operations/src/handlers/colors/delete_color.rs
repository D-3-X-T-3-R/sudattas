use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::colors;
use proto::proto::core::{ColorResponse, ColorsResponse, DeleteColorRequest};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_color(
    txn: &DatabaseTransaction,
    request: Request<DeleteColorRequest>,
) -> Result<Response<ColorsResponse>, Status> {
    let req = request.into_inner();

    let found = colors::Entity::find_by_id(req.color_id).one(txn).await;

    match found {
        Ok(Some(model)) => {
            match colors::Entity::delete_by_id(req.color_id).exec(txn).await {
                Ok(_) => Ok(Response::new(ColorsResponse {
                    items: vec![ColorResponse {
                        color_id: model.color_id,
                        color_name: model.color_name,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Color with ID {} not found",
            req.color_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
