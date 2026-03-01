use crate::handlers::db_errors::map_db_error_to_status;
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

    let model = users::ActiveModel {
        user_id: ActiveValue::Set(existing.user_id),
        username: ActiveValue::Set(req.username.unwrap_or(existing.username)),
        password: ActiveValue::Set(req.password.unwrap_or(existing.password)),
        email: ActiveValue::Set(req.email.unwrap_or(existing.email)),
        full_name: ActiveValue::Set(req.full_name.or(existing.full_name)),
        address: ActiveValue::Set(req.address.or(existing.address)),
        phone: ActiveValue::Set(req.phone.or(existing.phone)),
        create_date: ActiveValue::Set(existing.create_date),
        password_hash: ActiveValue::Set(existing.password_hash),
        email_verified: ActiveValue::Set(existing.email_verified),
        email_verified_at: ActiveValue::Set(existing.email_verified_at),
        user_status_id: ActiveValue::Set(existing.user_status_id),
        last_login_at: ActiveValue::Set(existing.last_login_at),
        marketing_opt_out: ActiveValue::Set(existing.marketing_opt_out),
        updated_at: ActiveValue::NotSet,
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(UsersResponse {
            items: vec![UserResponse {
                user_id: updated.user_id,
                username: updated.username,
                email: updated.email,
                full_name: updated.full_name,
                address: updated.address,
                phone: updated.phone,
                create_date: updated.create_date.to_rfc3339(),
                session_id: None,
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
