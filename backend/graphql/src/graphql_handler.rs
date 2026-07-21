//! Phase 8: Custom GraphQL handler with depth/complexity limits.
//!
//! Parses body, checks query depth, then executes via Juniper and returns JSON.
//! P1: Records request duration and outcome for Prometheus.

use crate::graphql_limits::{analyze_query, DEFAULT_MAX_QUERY_COMPLEXITY, DEFAULT_MAX_QUERY_DEPTH};
use crate::metrics;
use crate::query_handler::Context;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info};
use warp::http::StatusCode;
use warp::hyper::body::Bytes;
use warp::reply::Response;
use warp::Reply;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLRequestBody {
    query: Option<String>,
    operation_name: Option<String>,
    #[serde(default)]
    variables: Option<juniper::Variables>,
}

/// Max query depth from env or default.
fn max_query_depth() -> u32 {
    std::env::var("GRAPHQL_MAX_QUERY_DEPTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_QUERY_DEPTH)
}

/// Max query complexity from env, or the default. Enforced unconditionally (not opt-in):
/// GRAPHQL_MAX_QUERY_COMPLEXITY only overrides the threshold, it doesn't gate whether the
/// check runs, since a query that skips this check entirely could still be arbitrarily wide.
fn max_query_complexity() -> u64 {
    std::env::var("GRAPHQL_MAX_QUERY_COMPLEXITY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_QUERY_COMPLEXITY)
}

/// Handles a GraphQL request: parse body, check depth, execute, return JSON response.
pub async fn handle_graphql_request(
    ctx: Context,
    body: Bytes,
    schema_ref: Arc<crate::Schema>,
) -> Result<Response, warp::Rejection> {
    let req: GraphQLRequestBody = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            return Ok(depth_limit_error_response(400, "Invalid JSON body"));
        }
    };
    let q = req.query.unwrap_or_default();
    let query = q.trim();
    if query.is_empty() {
        return Ok(depth_limit_error_response(400, "Missing 'query' field"));
    }

    let metrics = match analyze_query(query) {
        Ok(m) => m,
        Err(e) => {
            return Ok(depth_limit_error_response(
                400,
                &format!("Invalid GraphQL query: {e}"),
            ));
        }
    };

    let max_depth = max_query_depth();
    if metrics.depth > max_depth {
        return Ok(depth_limit_error_response(
            400,
            &format!(
                "Query depth limit exceeded: depth {} exceeds maximum {}",
                metrics.depth, max_depth
            ),
        ));
    }

    let max_complexity = max_query_complexity();
    if metrics.complexity > max_complexity {
        return Ok(depth_limit_error_response(
            400,
            &format!(
                "Query complexity limit exceeded: score {} exceeds maximum {}",
                metrics.complexity, max_complexity
            ),
        ));
    }

    let operation_name = req.operation_name.as_deref();
    let variables = req.variables.unwrap_or_default();

    let start = Instant::now();
    let result =
        juniper::execute(query, operation_name, schema_ref.as_ref(), &variables, &ctx).await;
    let duration_sec = start.elapsed().as_secs_f64();
    metrics::record_graphql_request_duration_seconds(duration_sec);
    let success = result
        .as_ref()
        .map(|(_, errs)| errs.is_empty())
        .unwrap_or(false);
    metrics::record_graphql_request_total(success);

    let (status, body_json) = match result {
        Ok((value, errors)) => {
            info!(
                request_id = ?ctx.request_id(),
                client_action = ?ctx.client_action(),
                auth_mode = %ctx.auth_mode(),
                has_idempotency_key = ctx.idempotency_key().is_some(),
                operation_name = ?operation_name,
                graphql_error_count = errors.len(),
                "GraphQL request processed"
            );
            let data = serde_json::to_value(&value).unwrap_or(serde_json::Value::Null);
            let errs: Vec<serde_json::Value> = errors
                .iter()
                .map(|e| {
                    // juniper::ExecutionError implements Serialize directly, producing the spec
                    // {message, locations, path, extensions?} shape (extensions included whenever
                    // IntoFieldError attached one) — use that instead of a Debug dump.
                    serde_json::to_value(e)
                        .unwrap_or_else(|_| serde_json::json!({ "message": format!("{:?}", e) }))
                })
                .collect();
            let response = serde_json::json!({
                "data": data,
                "errors": if errs.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::Array(errs)
                }
            });
            (200, response)
        }
        Err(e) => {
            error!(
                request_id = ?ctx.request_id(),
                client_action = ?ctx.client_action(),
                auth_mode = %ctx.auth_mode(),
                has_idempotency_key = ctx.idempotency_key().is_some(),
                operation_name = ?operation_name,
                error = %e,
                "GraphQL execution failed"
            );
            let response = serde_json::json!({
                "data": null,
                "errors": [{ "message": e.to_string() }]
            });
            (200, response)
        }
    };

    let body_str = serde_json::to_string(&body_json).unwrap_or_else(|_| "{}".to_string());
    let status_code = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
    Ok(warp::reply::with_status(
        warp::reply::with_header(body_str, "content-type", "application/json"),
        status_code,
    )
    .into_response())
}

fn depth_limit_error_response(status: u16, message: &str) -> Response {
    let body = serde_json::json!({
        "errors": [{ "message": message }]
    });
    let body_str = serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string());
    let status_code = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST);
    warp::reply::with_status(
        warp::reply::with_header(body_str, "content-type", "application/json"),
        status_code,
    )
    .into_response()
}
