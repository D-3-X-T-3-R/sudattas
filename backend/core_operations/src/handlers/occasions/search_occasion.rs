use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::occasions;
use proto::proto::core::{OccasionResponse, OccasionsResponse, SearchOccasionRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_occasion(
    txn: &DatabaseTransaction,
    request: Request<SearchOccasionRequest>,
) -> Result<Response<OccasionsResponse>, Status> {
    let req = request.into_inner();

    let mut query = occasions::Entity::find();
    if req.occasion_id != 0 {
        query = query.filter(occasions::Column::OccasionId.eq(req.occasion_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| OccasionResponse {
                    occasion_id: m.occasion_id,
                    occasion_name: m.name,
                })
                .collect();
            Ok(Response::new(OccasionsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
