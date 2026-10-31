use super::user_status::{fetch_status_code_map, resolve_status_code};
use super::{gender_to_string, parse_date_of_birth, parse_gender};
use crate::auth;
use crate::handlers::db_errors::{map_auth_error_to_status, map_db_error_to_status};
use core_db_entities::entity::sea_orm_active_enums::AuthProvider;
use core_db_entities::entity::users;
use proto::proto::core::{UpdateUserRequest, UserResponse, UsersResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_user(
    txn: &DatabaseTransaction,
    request: Request<UpdateUserRequest>,
) -> Result<Response<UsersResponse>, Status> {
    let req = request.into_inner();

    let existing = users::Entity::find_by_id(req.user_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("User with ID {} not found", req.user_id)))?;

    let password_hash = match &existing.auth_provider {
        AuthProvider::Email => match req.password_plain {
            Some(ref plain) if !plain.trim().is_empty() => {
                let hash = auth::hash_password(plain).map_err(map_auth_error_to_status)?;
                Some(hash)
            }
            _ => existing.password_hash.clone(),
        },
        AuthProvider::Google => None,
    };

    let google_sub = match &existing.auth_provider {
        AuthProvider::Google => req.google_sub.or(existing.google_sub.clone()),
        AuthProvider::Email => None,
    };

    let auth_provider_str = match &existing.auth_provider {
        AuthProvider::Email => "email",
        AuthProvider::Google => "google",
    };

    let first_name = req.first_name.clone().or(existing.first_name.clone());
    let last_name = req.last_name.clone().or(existing.last_name.clone());

    let gender = match req.gender {
        Some(ref raw) => Some(parse_gender(raw)?),
        None => existing.gender.clone(),
    };

    let date_of_birth = match req.date_of_birth {
        Some(ref raw) => Some(parse_date_of_birth(raw)?),
        None => existing.date_of_birth,
    };

    // Derive full_name from first/last when the caller updates either one and doesn't
    // explicitly override full_name itself; keeps admin views, order emails, and invoices
    // (which all read full_name) in sync with the new first/last name fields.
    let full_name = if req.full_name.is_some() {
        req.full_name.clone()
    } else if req.first_name.is_some() || req.last_name.is_some() {
        let combined = [first_name.as_deref(), last_name.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        let trimmed = combined.trim();
        if trimmed.is_empty() {
            existing.full_name.clone()
        } else {
            Some(trimmed.to_string())
        }
    } else {
        existing.full_name.clone()
    };

    let model = users::ActiveModel {
        user_id: ActiveValue::Set(existing.user_id),
        username: ActiveValue::Set(req.username.unwrap_or(existing.username)),
        email: ActiveValue::Set(req.email.unwrap_or(existing.email)),
        auth_provider: ActiveValue::Set(existing.auth_provider),
        password_hash: ActiveValue::Set(password_hash),
        google_sub: ActiveValue::Set(google_sub),
        full_name: ActiveValue::Set(full_name),
        address: ActiveValue::Set(req.address.or(existing.address)),
        phone: ActiveValue::Set(req.phone.or(existing.phone)),
        first_name: ActiveValue::Set(first_name),
        last_name: ActiveValue::Set(last_name),
        gender: ActiveValue::Set(gender),
        date_of_birth: ActiveValue::Set(date_of_birth),
        create_date: ActiveValue::Set(existing.create_date),
        role_id: ActiveValue::Set(req.role_id.or(existing.role_id)),
        email_verified: ActiveValue::Set(existing.email_verified),
        email_verified_at: ActiveValue::Set(existing.email_verified_at),
        user_status_id: ActiveValue::Set(existing.user_status_id),
        last_login_at: ActiveValue::Set(existing.last_login_at),
        marketing_opt_out: ActiveValue::Set(existing.marketing_opt_out),
        updated_at: ActiveValue::NotSet,
    };

    match model.update(txn).await {
        Ok(updated) => {
            let status_map = fetch_status_code_map(txn).await?;
            Ok(Response::new(UsersResponse {
                items: vec![UserResponse {
                    user_id: updated.user_id,
                    username: updated.username,
                    email: updated.email,
                    auth_provider: auth_provider_str.to_string(),
                    full_name: updated.full_name,
                    address: updated.address,
                    phone: updated.phone,
                    create_date: updated.create_date.to_rfc3339(),
                    session_id: None,
                    role_id: updated.role_id,
                    user_status_id: updated.user_status_id,
                    first_name: updated.first_name,
                    last_name: updated.last_name,
                    gender: updated.gender.as_ref().map(gender_to_string),
                    date_of_birth: updated.date_of_birth.map(|d| d.to_string()),
                    user_status: resolve_status_code(&status_map, updated.user_status_id),
                }],
            }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
