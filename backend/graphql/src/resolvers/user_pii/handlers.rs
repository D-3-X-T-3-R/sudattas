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
    })
}
