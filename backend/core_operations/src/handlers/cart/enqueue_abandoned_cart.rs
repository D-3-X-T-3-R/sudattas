//! P2 Abandoned cart: RPC handler that calls the procedure (scheduler/cron).

use crate::procedures::abandoned_cart::{
    enqueue_abandoned_cart_events, DEFAULT_ABANDONED_DELAY_HOURS,
};
use proto::proto::core::{EnqueueAbandonedCartRequest, EnqueueAbandonedCartResponse};
use sea_orm::DatabaseConnection;
use tonic::{Request, Response, Status};

pub async fn enqueue_abandoned_cart(
    db: &DatabaseConnection,
    request: Request<EnqueueAbandonedCartRequest>,
) -> Result<Response<EnqueueAbandonedCartResponse>, Status> {
    let req = request.into_inner();
    let delay_hours = req.delay_hours.unwrap_or(DEFAULT_ABANDONED_DELAY_HOURS);
    let enqueued = enqueue_abandoned_cart_events(db, delay_hours).await?;
    Ok(Response::new(EnqueueAbandonedCartResponse {
        enqueued_count: enqueued as i32,
    }))
}
