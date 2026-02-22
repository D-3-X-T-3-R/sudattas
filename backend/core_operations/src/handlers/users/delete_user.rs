use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::users;
use proto::proto::core::{DeleteUserRequest, UserResponse, UsersResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_user(
    txn: &DatabaseTransaction,
    request: Request<DeleteUserRequest>,
) -> Result<Response<UsersResponse>, Status> {
    let req = request.into_inner();

    let found = users::Entity::find_by_id(req.user_id).one(txn).await;

    match found {
        Ok(Some(model)) => match users::Entity::delete_by_id(req.user_id).exec(txn).await {
            Ok(_) => Ok(Response::new(UsersResponse {
                items: vec![UserResponse {
                    user_id: model.user_id,
                    username: model.username,
                    email: model.email,
                    full_name: model.full_name,
                    address: model.address,
                    phone: model.phone,
                    create_date: model.create_date.to_rfc3339(),
                    session_id: None,
                }],
            })),
            Err(e) => Err(map_db_error_to_status(e)),
        },
        Ok(None) => Err(Status::not_found(format!(
            "User with ID {} not found",
            req.user_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
