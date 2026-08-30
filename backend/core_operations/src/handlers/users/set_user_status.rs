//! Admin: activate/deactivate/suspend a user account. Deliberately a dedicated mutation
//! (see `SetUserStatusRequest`'s doc comment in messages.proto) — `update_user` intentionally
//! never touches `user_status_id`.
//!
//! This is more than a database flag: the GraphQL layer's auth gate (`resolve_admin_from_db`
//! in `graphql/src/query_handler/admin_roles.rs`) checks the resolved status on every
//! JWT-authenticated request and rejects "inactive"/"suspended" accounts outright, so setting
//! this actually blocks the account — login, checkout, everything — not just a cosmetic label
//! (subject to that check's ~5 minute cache TTL).

use super::user_status::{
    fetch_status_code_map, normalize_status_code, resolve_status_code, status_id_for_code,
};
use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::sea_orm_active_enums::AuthProvider;
use core_db_entities::entity::users;
use proto::proto::core::{SetUserStatusRequest, UserResponse, UsersResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel};
use sea_orm::DatabaseTransaction;
use tonic::{Request, Response, Status};

pub async fn set_user_status(
    txn: &DatabaseTransaction,
    request: Request<SetUserStatusRequest>,
) -> Result<Response<UsersResponse>, Status> {
    let req = request.into_inner();

    let Some(code) = normalize_status_code(&req.status) else {
        return Err(Status::invalid_argument(
            "status must be \"active\", \"inactive\", or \"suspended\"",
        ));
    };
    let status_id = status_id_for_code(txn, code).await?;

    let existing = users::Entity::find_by_id(req.user_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| Status::not_found(format!("User with ID {} not found", req.user_id)))?;

    let auth_provider_str = match existing.auth_provider {
        AuthProvider::Google => "google",
        AuthProvider::Email => "email",
    };

    let mut active = existing.into_active_model();
    active.user_status_id = ActiveValue::Set(Some(status_id));

    let updated = active.update(txn).await.map_err(map_db_error_to_status)?;
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
            gender: updated.gender.as_ref().map(super::gender_to_string),
            date_of_birth: updated.date_of_birth.map(|d| d.to_string()),
            user_status: resolve_status_code(&status_map, updated.user_status_id),
        }],
    }))
}
