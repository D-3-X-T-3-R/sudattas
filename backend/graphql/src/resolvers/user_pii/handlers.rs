//! P2 Data retention: export my PII via gRPC GetUserPiiExport.

use crate::query_handler::Context;
use crate::resolvers::error::{Code, GqlError};
use crate::resolvers::user_pii::schema::UserPiiExport;
use crate::resolvers::utils::connect_grpc_client_from_context;
use proto::proto::core::GetUserPiiExportRequest;

pub async fn export_my_pii(context: &Context) -> Result<UserPiiExport, GqlError> {
    let user_id_str = context
        .user_id()
        .ok_or_else(|| GqlError::new("Must be logged in to export PII", Code::Unauthenticated))?;
    let user_id: i64 = user_id_str
        .parse()
        .map_err(|_| GqlError::new("Invalid user id", Code::InvalidArgument))?;

    fetch_pii_export(context, user_id).await
}

/// Admin lookup of any customer's PII by id — same gRPC call `export_my_pii` uses, just without
/// pinning `user_id` to the caller's own identity. The gRPC handler (`get_user_pii_export`) was
/// already written to serve either case ("Caller must be self or admin" in its own doc comment)
/// but nothing above it ever actually exercised the admin half until now: `export_my_pii` only
/// ever passed the caller's own id, so an admin had no way to look up someone else's record.
/// Authorization (`require_admin`) is enforced by the caller — the `adminExportUserPii` query
/// resolver in `query_root.rs` — matching every other admin-only query's convention.
pub async fn admin_export_user_pii(
    context: &Context,
    user_id: String,
) -> Result<UserPiiExport, GqlError> {
    let user_id: i64 = user_id
        .parse()
        .map_err(|_| GqlError::new("Invalid user id", Code::InvalidArgument))?;

    fetch_pii_export(context, user_id).await
}

async fn fetch_pii_export(context: &Context, user_id: i64) -> Result<UserPiiExport, GqlError> {
    let mut client = connect_grpc_client_from_context(context).await?;
    let response = client
        .get_user_pii_export(GetUserPiiExportRequest { user_id })
        .await
        .map_err(crate::resolvers::error::map_err)?
        .into_inner();

    Ok(UserPiiExport {
        user_id: response.user_id as i32,
        email: response.email,
        full_name: response.full_name,
        address: response.address,
        phone: response.phone,
        create_date: response.create_date,
        first_name: response.first_name,
        last_name: response.last_name,
        gender: response.gender,
        date_of_birth: response.date_of_birth,
    })
}
