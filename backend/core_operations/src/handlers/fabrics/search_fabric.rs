use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::fabrics;
use proto::proto::core::{FabricResponse, FabricsResponse, SearchFabricRequest};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_fabric(
    txn: &DatabaseTransaction,
    request: Request<SearchFabricRequest>,
) -> Result<Response<FabricsResponse>, Status> {
    let req = request.into_inner();

    let mut query = fabrics::Entity::find();
    if req.fabric_id != 0 {
        query = query.filter(fabrics::Column::FabricId.eq(req.fabric_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| FabricResponse {
                    fabric_id: m.fabric_id,
                    fabric_name: m.name,
                })
                .collect();
            Ok(Response::new(FabricsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
