use crate::auth;
use crate::handlers::db_errors::{map_auth_error_to_status, map_db_error_to_status};
use chrono::Utc;
use core_db_entities::entity::users;
use proto::proto::core::{CreateUserRequest, UserResponse, UsersResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_user(
    txn: &DatabaseTransaction,
    request: Request<CreateUserRequest>,
) -> Result<Response<UsersResponse>, Status> {
    let req = request.into_inner();

    auth::validate_password_strength(&req.password).map_err(map_auth_error_to_status)?;
    let password_hash = auth::hash_password(&req.password).map_err(map_auth_error_to_status)?;

    let model = users::ActiveModel {
        user_id: ActiveValue::NotSet,
        username: ActiveValue::Set(req.username),
        password: ActiveValue::Set(String::new()), // Legacy column; auth uses password_hash only
        email: ActiveValue::Set(req.email),
        full_name: ActiveValue::Set(req.full_name),
        address: ActiveValue::Set(req.address),
        phone: ActiveValue::Set(req.phone),
        create_date: ActiveValue::Set(Utc::now()),
        password_hash: ActiveValue::Set(Some(password_hash)),
        email_verified: ActiveValue::NotSet,
        email_verified_at: ActiveValue::NotSet,
        user_status_id: ActiveValue::NotSet,
        last_login_at: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(UsersResponse {
            items: vec![UserResponse {
                user_id: inserted.user_id,
                username: inserted.username,
                email: inserted.email,
                full_name: inserted.full_name,
                address: inserted.address,
                phone: inserted.phone,
                create_date: inserted.create_date.to_rfc3339(),
                session_id: None,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
