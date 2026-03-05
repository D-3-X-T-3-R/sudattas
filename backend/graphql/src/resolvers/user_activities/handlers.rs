use proto::proto::core::{
    CreateUserActivityRequest, DeleteUserActivityRequest, SearchUserActivityRequest,
    UpdateUserActivityRequest, UserActivitiesResponse, UserActivityResponse,
};
use tracing::instrument;

use super::schema::{
    DeleteUserActivityInput, NewUserActivity, SearchUserActivityInput, UserActivity,
    UserActivityMutation,
};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn activity_response_to_gql(a: UserActivityResponse) -> UserActivity {
    UserActivity {
        activity_id: a.activity_id.to_string(),
        user_id: a.user_id.to_string(),
        activity_type: a.activity_type,
        activity_time: a.activity_time,
        activity_details: a.activity_details,
    }
}

fn activities_response_to_vec(resp: UserActivitiesResponse) -> Vec<UserActivity> {
    resp.items
        .into_iter()
        .map(activity_response_to_gql)
        .collect()
}

#[instrument]
pub(crate) async fn create_user_activity(
    input: NewUserActivity,
) -> Result<Vec<UserActivity>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_user_activity(CreateUserActivityRequest {
            user_id: parse_i64(&input.user_id, "user_id")?,
            activity_type: input.activity_type,
            activity_details: input.activity_details,
        })
        .await?;
    Ok(activities_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_user_activity(
    input: SearchUserActivityInput,
) -> Result<Vec<UserActivity>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_user_activity(SearchUserActivityRequest {
            activity_id: parse_i64(&input.activity_id, "activity_id")?,
        })
        .await?;
    Ok(activities_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_user_activity(
    input: UserActivityMutation,
) -> Result<Vec<UserActivity>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_user_activity(UpdateUserActivityRequest {
            activity_id: parse_i64(&input.activity_id, "activity_id")?,
            user_id: input
                .user_id
                .as_deref()
                .map(|s| parse_i64(s, "user_id"))
                .transpose()?,
            activity_type: input.activity_type,
            activity_details: input.activity_details,
        })
        .await?;
    Ok(activities_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_user_activity(
    input: DeleteUserActivityInput,
) -> Result<Vec<UserActivity>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_user_activity(DeleteUserActivityRequest {
            activity_id: parse_i64(&input.activity_id, "activity_id")?,
        })
        .await?;
    Ok(activities_response_to_vec(resp.into_inner()))
}
