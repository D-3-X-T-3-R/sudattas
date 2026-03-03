//! ShippingZone resolvers: gRPC ShippingZone APIs were removed; schema kept for compatibility, returns empty.

use tracing::instrument;

use super::schema::{NewShippingZone, SearchShippingZone, ShippingZone, ShippingZoneMutation};
use crate::resolvers::error::GqlError;

#[instrument]
pub(crate) async fn search_shipping_zone(
    _input: SearchShippingZone,
) -> Result<Vec<ShippingZone>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn create_shipping_zone(
    _input: NewShippingZone,
) -> Result<Vec<ShippingZone>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn update_shipping_zone(
    _input: ShippingZoneMutation,
) -> Result<Vec<ShippingZone>, GqlError> {
    Ok(vec![])
}

#[instrument]
pub(crate) async fn delete_shipping_zone(_zone_id: String) -> Result<Vec<ShippingZone>, GqlError> {
    Ok(vec![])
}
