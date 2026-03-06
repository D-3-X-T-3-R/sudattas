use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::weaves;
use proto::proto::core::{SearchWeaveRequest, WeaveResponse, WeavesResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_weave(
    txn: &DatabaseTransaction,
    request: Request<SearchWeaveRequest>,
) -> Result<Response<WeavesResponse>, Status> {
    let req = request.into_inner();

    let mut query = weaves::Entity::find();
    if req.weave_id != 0 {
        query = query.filter(weaves::Column::WeaveId.eq(req.weave_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| WeaveResponse {
                    weave_id: m.weave_id,
                    weave_name: m.name,
                })
                .collect();
            Ok(Response::new(WeavesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
