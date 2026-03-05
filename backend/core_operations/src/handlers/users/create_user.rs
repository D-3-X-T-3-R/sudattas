use crate::auth;
use crate::handlers::db_errors::{map_auth_error_to_status, map_db_error_to_status};
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::AuthProvider;
use core_db_entities::entity::users;
use proto::proto::core::{CreateUserRequest, UserResponse, UsersResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_user(
    txn: &DatabaseTransaction,
    request: Request<CreateUserRequest>,
) -> Result<Response<UsersResponse>, Status> {
    let req = request.into_inner();

    let (auth_provider, password_hash, google_sub) = match req.auth_provider.as_str() {
        "google" => {
            let sub = req
                .google_sub
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| {
                    Status::invalid_argument("google_sub is required for google auth")
                })?;
            (AuthProvider::Google, None, Some(sub))
        }
        "email" | "" => {
            let plain = req
                .password_plain
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| {
                    Status::invalid_argument("password_plain is required for email auth")
                })?;
            auth::validate_password_strength(&plain).map_err(map_auth_error_to_status)?;
            let hash = auth::hash_password(&plain).map_err(map_auth_error_to_status)?;
            (AuthProvider::Email, Some(hash), None)
        }
        other => {
            return Err(Status::invalid_argument(format!(
                "unknown auth_provider '{}'; expected 'email' or 'google'",
                other
            )))
        }
    };

    let auth_provider_str = match &auth_provider {
        AuthProvider::Email => "email",
        AuthProvider::Google => "google",
    };

    let model = users::ActiveModel {
        user_id: ActiveValue::NotSet,
        username: ActiveValue::Set(req.username),
        email: ActiveValue::Set(req.email),
        auth_provider: ActiveValue::Set(auth_provider),
        password_hash: ActiveValue::Set(password_hash),
        google_sub: ActiveValue::Set(google_sub),
        full_name: ActiveValue::Set(req.full_name),
        address: ActiveValue::Set(req.address),
        phone: ActiveValue::Set(req.phone),
        create_date: ActiveValue::Set(Utc::now()),
        role_id: ActiveValue::Set(req.role_id),
        email_verified: ActiveValue::NotSet,
        email_verified_at: ActiveValue::NotSet,
        user_status_id: ActiveValue::NotSet,
        last_login_at: ActiveValue::NotSet,
        marketing_opt_out: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(UsersResponse {
            items: vec![UserResponse {
                user_id: inserted.user_id,
                username: inserted.username,
                email: inserted.email,
                auth_provider: auth_provider_str.to_string(),
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
