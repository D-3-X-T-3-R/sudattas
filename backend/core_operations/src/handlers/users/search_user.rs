use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::sea_orm_active_enums::AuthProvider;
use core_db_entities::entity::users;
use proto::proto::core::{SearchUserRequest, UserResponse, UsersResponse};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use tonic::{Request, Response, Status};

pub async fn search_user(
    txn: &DatabaseTransaction,
    request: Request<SearchUserRequest>,
) -> Result<Response<UsersResponse>, Status> {
    let req = request.into_inner();

    let mut query = users::Entity::find();

    if let Some(user_id) = req.user_id {
        if user_id > 0 {
            query = query.filter(users::Column::UserId.eq(user_id));
        }
    }
    if let Some(username) = req.username.filter(|v| !v.trim().is_empty()) {
        query = query.filter(users::Column::Username.eq(username));
    }
    if let Some(email) = req.email.filter(|v| !v.trim().is_empty()) {
        query = query.filter(users::Column::Email.eq(email));
    }
    if let Some(auth_provider) = req.auth_provider.filter(|v| !v.trim().is_empty()) {
        let auth = match auth_provider.trim().to_lowercase().as_str() {
            "google" => Some(AuthProvider::Google),
            "email" => Some(AuthProvider::Email),
            _ => None,
        };
        if let Some(auth) = auth {
            query = query.filter(users::Column::AuthProvider.eq(auth));
        }
    }
    if let Some(google_sub) = req.google_sub.filter(|v| !v.trim().is_empty()) {
        query = query.filter(users::Column::GoogleSub.eq(google_sub));
    }
    if let Some(full_name) = req.full_name.filter(|v| !v.trim().is_empty()) {
        query = query.filter(users::Column::FullName.eq(full_name));
    }
    if let Some(address) = req.address.filter(|v| !v.trim().is_empty()) {
        query = query.filter(users::Column::Address.eq(address));
    }
    if let Some(phone) = req.phone.filter(|v| !v.trim().is_empty()) {
        query = query.filter(users::Column::Phone.eq(phone));
    }
    if let Some(role_id) = req.role_id {
        if role_id > 0 {
            query = query.filter(users::Column::RoleId.eq(role_id));
        }
    }
    if let Some(user_status_id) = req.user_status_id {
        if user_status_id > 0 {
            query = query.filter(users::Column::UserStatusId.eq(user_status_id));
        }
    }

    query = query.order_by_desc(users::Column::UserId);

    let limit = req.limit.unwrap_or(50).clamp(1, 200) as u64;
    let offset = req.offset.unwrap_or(0).max(0) as u64;
    query = query.limit(limit).offset(offset);

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| {
                    let ap = match m.auth_provider {
                        AuthProvider::Google => "google",
                        AuthProvider::Email => "email",
                    };
                    UserResponse {
                        user_id: m.user_id,
                        username: m.username,
                        email: m.email,
                        auth_provider: ap.to_string(),
                        full_name: m.full_name,
                        address: m.address,
                        phone: m.phone,
                        create_date: m.create_date.to_rfc3339(),
                        session_id: None,
                        role_id: m.role_id,
                        user_status_id: m.user_status_id,
                    }
                })
                .collect();
            Ok(Response::new(UsersResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
