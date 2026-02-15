use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_roles;
use proto::proto::core::{UpdateUserRoleRequest, UserRoleResponse, UserRolesResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn update_user_role(
    txn: &DatabaseTransaction,
    request: Request<UpdateUserRoleRequest>,
) -> Result<Response<UserRolesResponse>, Status> {
    let req = request.into_inner();
    let model = user_roles::ActiveModel {
        role_id: ActiveValue::Set(req.role_id),
        role_name: ActiveValue::Set(req.role_name),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(UserRolesResponse {
            items: vec![UserRoleResponse {
                role_id: updated.role_id,
                role_name: updated.role_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
