use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_roles;
use proto::proto::core::{SearchUserRoleRequest, UserRoleResponse, UserRolesResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn search_user_role(
    txn: &DatabaseTransaction,
    request: Request<SearchUserRoleRequest>,
) -> Result<Response<UserRolesResponse>, Status> {
    let req = request.into_inner();

    let mut query = user_roles::Entity::find();
    if req.role_id != 0 {
        query = query.filter(user_roles::Column::RoleId.eq(req.role_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| UserRoleResponse {
                    role_id: m.role_id,
                    role_name: m.role_name,
                })
                .collect();
            Ok(Response::new(UserRolesResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
