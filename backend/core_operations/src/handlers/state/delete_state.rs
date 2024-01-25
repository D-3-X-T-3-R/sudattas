use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::states;
use proto::proto::core::{DeleteStateRequest, StateResponse, StatesResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_state(
    txn: &DatabaseTransaction,
    request: Request<DeleteStateRequest>,
) -> Result<Response<StatesResponse>, Status> {
    let req = request.into_inner();

    let state = states::Entity::find_by_id(req.state_id).one(txn).await;

    match state {
        Ok(Some(model)) => {
            match states::Entity::delete_many()
                .filter(states::Column::StateId.eq(req.state_id))
                .exec(txn)
                .await
            {
                Ok(delete_result) => {
                    if delete_result.rows_affected > 0 {
                        let response = StatesResponse {
                            items: vec![StateResponse {
                                state_name: model.state_name.unwrap(),
                                state_id: model.state_id,
                            }],
                        };
                        Ok(Response::new(response))
                    } else {
                        Err(Status::not_found(format!(
                            "State with ID {} not found.",
                            req.state_id
                        )))
                    }
                }
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "State with ID {} not found.",
            req.state_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
