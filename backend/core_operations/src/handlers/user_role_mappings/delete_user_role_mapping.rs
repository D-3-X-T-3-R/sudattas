use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_role_mapping;
use proto::proto::core::{
    DeleteUserRoleMappingRequest, UserRoleMappingResponse, UserRoleMappingsResponse,
};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn delete_user_role_mapping(
    txn: &DatabaseTransaction,
    request: Request<DeleteUserRoleMappingRequest>,
) -> Result<Response<UserRoleMappingsResponse>, Status> {
    let req = request.into_inner();

    let found = user_role_mapping::Entity::find()
        .filter(user_role_mapping::Column::UserId.eq(req.user_id))
        .filter(user_role_mapping::Column::RoleId.eq(req.role_id))
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match user_role_mapping::Entity::delete_many()
                .filter(user_role_mapping::Column::UserId.eq(req.user_id))
                .filter(user_role_mapping::Column::RoleId.eq(req.role_id))
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(UserRoleMappingsResponse {
                    items: vec![UserRoleMappingResponse {
                        user_id: model.user_id,
                        role_id: model.role_id,
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found("UserRoleMapping not found")),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
