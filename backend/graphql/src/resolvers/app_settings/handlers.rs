use proto::proto::core::{
    AbandonedCartSettingsResponse, GetAbandonedCartSettingsRequest,
    UpdateAbandonedCartSettingsRequest,
};
use tracing::instrument;

use super::schema::{AbandonedCartSettings, UpdateAbandonedCartSettingsInput};
use crate::resolvers::{
    error::GqlError,
    utils::{connect_grpc_client, parse_i64},
};

fn settings_response_to_gql(row: AbandonedCartSettingsResponse) -> AbandonedCartSettings {
    AbandonedCartSettings {
        enabled: row.enabled,
        delay_hours: row.delay_hours.to_string(),
        poll_interval_sec: row.poll_interval_sec.to_string(),
    }
}

#[instrument]
pub(crate) async fn abandoned_cart_settings() -> Result<AbandonedCartSettings, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .get_abandoned_cart_settings(GetAbandonedCartSettingsRequest {})
        .await?;
    Ok(settings_response_to_gql(response.into_inner()))
}

#[instrument]
pub(crate) async fn update_abandoned_cart_settings(
    input: UpdateAbandonedCartSettingsInput,
) -> Result<AbandonedCartSettings, GqlError> {
    let mut client = connect_grpc_client().await?;
    let response = client
        .update_abandoned_cart_settings(UpdateAbandonedCartSettingsRequest {
            enabled: input.enabled,
            delay_hours: input
                .delay_hours
                .as_deref()
                .map(|v| parse_i64(v, "delay_hours"))
                .transpose()?,
            poll_interval_sec: input
                .poll_interval_sec
                .as_deref()
                .map(|v| parse_i64(v, "poll_interval_sec"))
                .transpose()?,
        })
        .await?;
    Ok(settings_response_to_gql(response.into_inner()))
}
