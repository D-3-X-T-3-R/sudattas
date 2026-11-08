use super::user_status::{fetch_status_code_map, resolve_status_code};
use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::sea_orm_active_enums::AuthProvider;
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
            Ok(_) => {
                let ap = match model.auth_provider {
                    AuthProvider::Google => "google",
                    AuthProvider::Email => "email",
                };
                let status_map = fetch_status_code_map(txn).await?;
                Ok(Response::new(UsersResponse {
                    items: vec![UserResponse {
                        user_id: model.user_id,
                        username: model.username,
                        email: model.email,
                        auth_provider: ap.to_string(),
                        full_name: model.full_name,
                        address: model.address,
                        phone: model.phone,
                        create_date: model.create_date.to_rfc3339(),
                        session_id: None,
                        role_id: model.role_id,
                        user_status_id: model.user_status_id,
                        first_name: model.first_name,
                        last_name: model.last_name,
                        gender: model.gender.as_ref().map(super::gender_to_string),
                        date_of_birth: model.date_of_birth.map(|d| d.to_string()),
                        user_status: resolve_status_code(&status_map, model.user_status_id),
                    }],
                }))
            }
            Err(e) => Err(map_db_error_to_status(e)),
        },
        Ok(None) => Err(Status::not_found(format!(
            "User with ID {} not found",
            req.user_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
