use proto::proto::core::{
    CreateUserRoleRequest, DeleteUserRoleRequest, SearchUserRoleRequest, UpdateUserRoleRequest,
    UserRoleResponse, UserRolesResponse,
};
use tracing::instrument;

use super::schema::{
    DeleteUserRoleInput, NewUserRole, SearchUserRoleInput, UserRole, UserRoleMutation,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn role_response_to_gql(r: UserRoleResponse) -> UserRole {
    UserRole {
        role_id: r.role_id.to_string(),
        role_name: r.role_name,
    }
}

fn roles_response_to_vec(resp: UserRolesResponse) -> Vec<UserRole> {
    resp.items.into_iter().map(role_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_user_role(input: NewUserRole) -> Result<Vec<UserRole>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_user_role(CreateUserRoleRequest {
            role_name: input.role_name,
        })
        .await?;
    Ok(roles_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_user_role(
    input: SearchUserRoleInput,
) -> Result<Vec<UserRole>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_user_role(SearchUserRoleRequest {
            role_id: parse_i64(&input.role_id, "role_id")?,
        })
        .await?;
    Ok(roles_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_user_role(input: UserRoleMutation) -> Result<Vec<UserRole>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_user_role(UpdateUserRoleRequest {
            role_id: parse_i64(&input.role_id, "role_id")?,
            role_name: input.role_name,
        })
        .await?;
    Ok(roles_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_user_role(
    input: DeleteUserRoleInput,
) -> Result<Vec<UserRole>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_user_role(DeleteUserRoleRequest {
            role_id: parse_i64(&input.role_id, "role_id")?,
        })
        .await?;
    Ok(roles_response_to_vec(resp.into_inner()))
}
