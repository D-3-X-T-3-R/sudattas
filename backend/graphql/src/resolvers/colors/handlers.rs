use proto::proto::core::{
    ColorResponse, ColorsResponse, CreateColorRequest, DeleteColorRequest, SearchColorRequest,
    UpdateColorRequest,
};
use tracing::instrument;

use super::schema::{Color, ColorMutation, DeleteColorInput, NewColor, SearchColorInput};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn color_response_to_gql(c: ColorResponse) -> Color {
    Color {
        color_id: c.color_id.to_string(),
        color_name: c.color_name,
    }
}

fn colors_response_to_vec(resp: ColorsResponse) -> Vec<Color> {
    resp.items.into_iter().map(color_response_to_gql).collect()
}

#[instrument]
pub(crate) async fn create_color(input: NewColor) -> Result<Vec<Color>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .create_color(CreateColorRequest {
            color_name: input.color_name,
        })
        .await?;
    Ok(colors_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn search_color(input: SearchColorInput) -> Result<Vec<Color>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .search_color(SearchColorRequest {
            color_id: parse_i64(&input.color_id, "color_id")?,
        })
        .await?;
    Ok(colors_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn update_color(input: ColorMutation) -> Result<Vec<Color>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .update_color(UpdateColorRequest {
            color_id: parse_i64(&input.color_id, "color_id")?,
            color_name: input.color_name,
        })
        .await?;
    Ok(colors_response_to_vec(resp.into_inner()))
}

#[instrument]
pub(crate) async fn delete_color(input: DeleteColorInput) -> Result<Vec<Color>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let resp = client
        .delete_color(DeleteColorRequest {
            color_id: parse_i64(&input.color_id, "color_id")?,
        })
        .await?;
    Ok(colors_response_to_vec(resp.into_inner()))
}
