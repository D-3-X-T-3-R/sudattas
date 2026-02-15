use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::colors;
use proto::proto::core::{ColorResponse, ColorsResponse, UpdateColorRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_color(
    txn: &DatabaseTransaction,
    request: Request<UpdateColorRequest>,
) -> Result<Response<ColorsResponse>, Status> {
    let req = request.into_inner();
    let model = colors::ActiveModel {
        color_id: ActiveValue::Set(req.color_id),
        color_name: ActiveValue::Set(req.color_name),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(ColorsResponse {
            items: vec![ColorResponse {
                color_id: updated.color_id,
                color_name: updated.color_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
