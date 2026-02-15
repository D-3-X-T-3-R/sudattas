use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_roles;
use proto::proto::core::{CreateUserRoleRequest, UserRoleResponse, UserRolesResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_user_role(
    txn: &DatabaseTransaction,
    request: Request<CreateUserRoleRequest>,
) -> Result<Response<UserRolesResponse>, Status> {
    let req = request.into_inner();
    let model = user_roles::ActiveModel {
        role_id: ActiveValue::NotSet,
        role_name: ActiveValue::Set(req.role_name),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(UserRolesResponse {
            items: vec![UserRoleResponse {
                role_id: inserted.role_id,
                role_name: inserted.role_name,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
