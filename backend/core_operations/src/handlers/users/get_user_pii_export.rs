//! P2 Data retention: export PII for a user (no password). Caller must be self or admin.

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::users;
use proto::proto::core::{GetUserPiiExportRequest, GetUserPiiExportResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn get_user_pii_export(
    txn: &DatabaseTransaction,
    request: Request<GetUserPiiExportRequest>,
) -> Result<Response<GetUserPiiExportResponse>, Status> {
    let req = request.into_inner();
    let user = users::Entity::find_by_id(req.user_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let model = user.ok_or_else(|| Status::not_found(format!("User {} not found", req.user_id)))?;

    Ok(Response::new(GetUserPiiExportResponse {
        user_id: model.user_id,
        email: model.email,
        full_name: model.full_name,
        address: model.address,
        phone: model.phone,
        create_date: model.create_date.to_rfc3339(),
    }))
}
