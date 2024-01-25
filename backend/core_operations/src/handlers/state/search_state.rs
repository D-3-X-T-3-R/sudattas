use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::states;
use proto::proto::core::{SearchStateRequest, StateResponse, StatesResponse};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_state(
    txn: &DatabaseTransaction,
    request: Request<SearchStateRequest>,
) -> Result<Response<StatesResponse>, Status> {
    let req = request.into_inner();

    match states::Entity::find()
        .apply_if(req.state_id, |query, v| {
            query.filter(states::Column::StateId.eq(v))
        })
        .apply_if(req.state_name, |query, v| {
            query.filter(states::Column::StateName.contains(v))
        })
        .all(txn)
        .await
    {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|model| StateResponse {
                    state_name: model.state_name.unwrap(),
                    state_id: model.state_id,
                })
                .collect();

            let response = StatesResponse { items };
            Ok(Response::new(response))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
