//! Phase 8: Custom GraphQL handler with depth/complexity limits.
//!
//! Parses body, checks query depth, then executes via Juniper and returns JSON.
//! P1: Records request duration and outcome for Prometheus.

use crate::graphql_limits::{check_query_complexity, check_query_depth, DEFAULT_MAX_QUERY_DEPTH};
use crate::metrics;
use crate::query_handler::Context;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use warp::http::StatusCode;
use warp::hyper::body::Bytes;
use warp::reply::Response;
use warp::Reply;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLRequestBody {
    query: Option<String>,
    operation_name: Option<String>,
    #[allow(dead_code)]
    variables: Option<serde_json::Value>,
}

/// Max query depth from env or default.
fn max_query_depth() -> u32 {
    std::env::var("GRAPHQL_MAX_QUERY_DEPTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_QUERY_DEPTH)
}

/// Max query complexity from env. If not set, complexity check is skipped (optional).
fn max_query_complexity() -> Option<u64> {
    std::env::var("GRAPHQL_MAX_QUERY_COMPLEXITY")
        .ok()
        .and_then(|s| s.parse().ok())
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

    let max_depth = max_query_depth();
    if let Err(msg) = check_query_depth(query, max_depth) {
        return Ok(depth_limit_error_response(400, &msg));
    }

    if let Some(max_complexity) = max_query_complexity() {
        if let Err(msg) = check_query_complexity(query, max_complexity) {
            return Ok(depth_limit_error_response(400, &msg));
        }
    }

    let operation_name = req.operation_name.as_deref();
    let variables = juniper::Variables::new();

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
            let data = serde_json::to_value(&value).unwrap_or(serde_json::Value::Null);
            let errs: Vec<serde_json::Value> = errors
                .iter()
                .map(|e| {
                    // ExecutionError may not implement Display; use debug or field if available
                    let msg = format!("{:?}", e);
                    serde_json::json!({ "message": msg })
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
