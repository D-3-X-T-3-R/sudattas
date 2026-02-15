use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_roles;
use proto::proto::core::{DeleteUserRoleRequest, UserRoleResponse, UserRolesResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_user_role(
    txn: &DatabaseTransaction,
    request: Request<DeleteUserRoleRequest>,
) -> Result<Response<UserRolesResponse>, Status> {
    let req = request.into_inner();

    let found = user_roles::Entity::find_by_id(req.role_id).one(txn).await;

    match found {
        Ok(Some(model)) => {
            match user_roles::Entity::delete_by_id(req.role_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(UserRolesResponse {
                    items: vec![UserRoleResponse {
                        role_id: model.role_id,
                        role_name: model.role_name,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "UserRole with ID {} not found",
            req.role_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
