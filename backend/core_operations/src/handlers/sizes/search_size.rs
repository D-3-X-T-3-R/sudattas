use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::sizes;
use proto::proto::core::{SearchSizeRequest, SizeResponse, SizesResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_size(
    txn: &DatabaseTransaction,
    request: Request<SearchSizeRequest>,
) -> Result<Response<SizesResponse>, Status> {
    let req = request.into_inner();

    let mut query = sizes::Entity::find();
    if req.size_id != 0 {
        query = query.filter(sizes::Column::SizeId.eq(req.size_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| SizeResponse {
                    size_id: m.size_id,
                    size_name: m.size_name,
                })
                .collect();
            Ok(Response::new(SizesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
