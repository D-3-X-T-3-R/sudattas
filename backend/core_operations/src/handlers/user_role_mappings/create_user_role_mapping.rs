use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::user_role_mapping;
use proto::proto::core::{
    CreateUserRoleMappingRequest, UserRoleMappingResponse, UserRoleMappingsResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_user_role_mapping(
    txn: &DatabaseTransaction,
    request: Request<CreateUserRoleMappingRequest>,
) -> Result<Response<UserRoleMappingsResponse>, Status> {
    let req = request.into_inner();
    let model = user_role_mapping::ActiveModel {
        user_id: ActiveValue::Set(req.user_id),
        role_id: ActiveValue::Set(req.role_id),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(UserRoleMappingsResponse {
            items: vec![UserRoleMappingResponse {
                user_id: inserted.user_id,
                role_id: inserted.role_id,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
