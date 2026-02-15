use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::colors;
use proto::proto::core::{ColorResponse, ColorsResponse, CreateColorRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_color(
    txn: &DatabaseTransaction,
    request: Request<CreateColorRequest>,
) -> Result<Response<ColorsResponse>, Status> {
    let req = request.into_inner();
    let model = colors::ActiveModel {
        color_id: ActiveValue::NotSet,
        color_name: ActiveValue::Set(req.color_name),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(ColorsResponse {
            items: vec![ColorResponse {
                color_id: inserted.color_id,
                color_name: inserted.color_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
