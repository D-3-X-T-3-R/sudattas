use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::states;
use proto::proto::core::{CreateStateRequest, StateResponse, StatesResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_state(
    txn: &DatabaseTransaction,
    request: Request<CreateStateRequest>,
) -> Result<Response<StatesResponse>, Status> {
    let req = request.into_inner();
    let state = states::ActiveModel {
        state_id: ActiveValue::NotSet,
        state_name: ActiveValue::Set(Some(req.state_name)),
    };
    match state.insert(txn).await {
        Ok(model) => {
            let response = StatesResponse {
                items: vec![StateResponse {
                    state_name: model.state_name.unwrap(),
                    state_id: model.state_id,
                }],
            };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
