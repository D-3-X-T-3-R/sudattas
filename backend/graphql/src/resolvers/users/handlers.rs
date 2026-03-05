use proto::proto::core::{
    CreateUserRequest, DeleteUserRequest, RecordSecurityAuditRequest, SearchUserRequest,
    UpdateUserRequest, UserResponse,
};
use tracing::instrument;

use super::schema::{
    DeleteUserInput, NewUser, RecordSecurityAuditEventInput, SearchUserInput, UpdateUserInput, User,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_option_i64},
};

fn user_response_to_gql(u: UserResponse) -> User {
    User {
        user_id: u.user_id.to_string(),
        username: u.username,
        email: u.email,
        auth_provider: u.auth_provider,
        full_name: u.full_name,
        address: u.address,
        phone: u.phone,
        create_date: u.create_date,
    }
}

#[instrument]
pub(crate) async fn create_user(input: NewUser) -> Result<Vec<User>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .create_user(CreateUserRequest {
            username: input.username,
            email: input.email,
            auth_provider: input.auth_provider,
            password_plain: input.password_plain,
            google_sub: input.google_sub,
            full_name: input.full_name,
            address: input.address,
            phone: input.phone,
            role_id: to_option_i64(input.role_id),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(user_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_user(input: SearchUserInput) -> Result<Vec<User>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let user_id = parse_i64(&input.user_id, "user_id")?;
    let response = client.search_user(SearchUserRequest { user_id }).await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(user_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_user(input: UpdateUserInput) -> Result<Vec<User>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let user_id = parse_i64(&input.user_id, "user_id")?;
    let response = client
        .update_user(UpdateUserRequest {
            user_id,
            username: input.username,
            email: input.email,
            password_plain: input.password_plain,
            google_sub: input.google_sub,
            full_name: input.full_name,
            address: input.address,
            phone: input.phone,
            role_id: to_option_i64(input.role_id),
        })
        .await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(user_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_user(input: DeleteUserInput) -> Result<Vec<User>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let user_id = parse_i64(&input.user_id, "user_id")?;
    let response = client.delete_user(DeleteUserRequest { user_id }).await?;
    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(user_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn record_security_audit_event(
    input: RecordSecurityAuditEventInput,
) -> Result<bool, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .record_security_audit_event(RecordSecurityAuditRequest {
            event_type: input.event_type,
            details: input.details,
        })
        .await?;
    Ok(response.into_inner().success)
}
