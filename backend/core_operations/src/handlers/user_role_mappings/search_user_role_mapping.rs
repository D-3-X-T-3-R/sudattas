use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_role_mapping;
use proto::proto::core::{
    SearchUserRoleMappingRequest, UserRoleMappingResponse, UserRoleMappingsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_user_role_mapping(
    txn: &DatabaseTransaction,
    request: Request<SearchUserRoleMappingRequest>,
) -> Result<Response<UserRoleMappingsResponse>, Status> {
    let req = request.into_inner();

    let mut query = user_role_mapping::Entity::find();
    query = query.filter(user_role_mapping::Column::UserId.eq(req.user_id));
    query = query.filter(user_role_mapping::Column::RoleId.eq(req.role_id));

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| UserRoleMappingResponse {
                    user_id: m.user_id,
                    role_id: m.role_id,
                })
                .collect();
            Ok(Response::new(UserRoleMappingsResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
