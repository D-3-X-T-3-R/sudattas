use super::user_status::{fetch_status_code_map, resolve_status_code};
use crate::auth;
use crate::handlers::db_errors::{map_auth_error_to_status, map_db_error_to_status};
use chrono::Utc;
use core_db_entities::entity::sea_orm_active_enums::AuthProvider;
use core_db_entities::entity::users;
use proto::proto::core::{CreateUserRequest, UserResponse, UsersResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter,
};
use std::collections::HashMap;
use tonic::{Request, Response, Status};

fn user_to_response(user: users::Model, status_map: &HashMap<i64, String>) -> UserResponse {
    let auth_provider_str = match user.auth_provider {
        AuthProvider::Email => "email",
        AuthProvider::Google => "google",
    };
    UserResponse {
        user_id: user.user_id,
        username: user.username,
        email: user.email,
        auth_provider: auth_provider_str.to_string(),
        full_name: user.full_name,
        address: user.address,
        phone: user.phone,
        create_date: user.create_date.to_rfc3339(),
        session_id: None,
        role_id: user.role_id,
        user_status_id: user.user_status_id,
        first_name: user.first_name,
        last_name: user.last_name,
        gender: user.gender.as_ref().map(super::gender_to_string),
        date_of_birth: user.date_of_birth.map(|d| d.to_string()),
        user_status: resolve_status_code(status_map, user.user_status_id),
    }
}

async fn find_existing_google_identity(
    txn: &DatabaseTransaction,
    email: &str,
    google_sub: Option<&str>,
    phone: Option<&str>,
) -> Result<Option<users::Model>, Status> {
    if let Some(sub) = google_sub {
        let by_sub = users::Entity::find()
            .filter(users::Column::GoogleSub.eq(sub))
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?;
        if by_sub.is_some() {
            return Ok(by_sub);
        }
    }

    let by_email = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;
    if by_email.is_some() {
        return Ok(by_email);
    }

    if let Some(p) = phone {
        let by_phone = users::Entity::find()
            .filter(users::Column::Phone.eq(p))
            .one(txn)
            .await
            .map_err(map_db_error_to_status)?;
        if by_phone.is_some() {
            return Ok(by_phone);
        }
    }

    Ok(None)
}

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

    let model = users::ActiveModel {
        user_id: ActiveValue::NotSet,
        username: ActiveValue::Set(req.username),
        email: ActiveValue::Set(req.email.clone()),
        auth_provider: ActiveValue::Set(auth_provider.clone()),
        password_hash: ActiveValue::Set(password_hash),
        google_sub: ActiveValue::Set(google_sub.clone()),
        full_name: ActiveValue::Set(req.full_name),
        address: ActiveValue::Set(req.address),
        phone: ActiveValue::Set(req.phone.clone()),
        first_name: ActiveValue::NotSet,
        last_name: ActiveValue::NotSet,
        gender: ActiveValue::NotSet,
        date_of_birth: ActiveValue::NotSet,
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
        Ok(inserted) => {
            let status_map = fetch_status_code_map(txn).await?;
            Ok(Response::new(UsersResponse {
                items: vec![user_to_response(inserted, &status_map)],
            }))
        }
        Err(e) => {
            // Google sign-in provisioning must be idempotent across retries and
            // mixed auth attempts (same email/phone/google_sub). On uniqueness
            // conflicts, resolve and return the existing canonical user instead
            // of creating drift through parallel inserts.
            if auth_provider == AuthProvider::Google {
                let existing = find_existing_google_identity(
                    txn,
                    req.email.as_str(),
                    google_sub.as_deref(),
                    req.phone.as_deref(),
                )
                .await?;
                if let Some(user) = existing {
                    let status_map = fetch_status_code_map(txn).await?;
                    return Ok(Response::new(UsersResponse {
                        items: vec![user_to_response(user, &status_map)],
                    }));
                }
            }
            Err(map_db_error_to_status(e))
        }
    }
}
