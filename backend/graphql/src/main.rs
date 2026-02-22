use crate::query_handler::{AuthSource, Context};
use crate::security::jwt_validator::validate_token;
use dotenv::dotenv;
use juniper::{EmptySubscription, RootNode};
use reqwest::StatusCode;
use tracing::{debug, info, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use warp::{http::Response, reply, Filter, Rejection, Reply};

mod query_handler;
mod resolvers;
mod security;
mod webhooks;

#[derive(Debug)]
struct Unauthorized {}
impl warp::reject::Reject for Unauthorized {}

type Schema = RootNode<
    'static,
    query_handler::query_root::QueryRoot,
    query_handler::mutation_root::MutationRoot,
    EmptySubscription<Context>,
>;

fn schema() -> Schema {
    RootNode::new(
        query_handler::query_root::QueryRoot {},
        query_handler::mutation_root::MutationRoot {},
        EmptySubscription::<Context>::new(),
    )
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_level(true)
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .json()
        .init();

    let jwks = security::jwks_loader::load_jwks()
        .await
        .expect("Failed to load JWKS");

    let redis_url = std::env::var("REDIS_URL").ok();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_credentials(true)
        .allow_headers(vec!["content-type", "authorization", "x-session-id"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    let load_balancer_health_check = warp::get().and(warp::path::end()).map(|| {
        Response::builder()
            .header("content-type", "text/plain")
            .body("OK")
    });

    // Per-request context filter.
    //
    // Builds a `Context` that includes the authenticated identity for every request:
    //   - JWT Bearer  → `AuthSource::Jwt(sub)`       — full login
    //   - X-Session-Id → `AuthSource::Session(uid)`  — guest session
    //   - Neither valid → 401 Unauthorized
    //
    // Resolvers inspect `context.jwt_user_id()` to gate operations that require a
    // full login (e.g. checkout / place_order).
    let jwks_c = jwks.clone();
    let redis_url_c = redis_url.clone();
    let context_filter = warp::header::optional::<String>("authorization")
        .and(warp::header::optional::<String>("x-session-id"))
        .and(warp::any().map(move || (jwks_c.clone(), redis_url_c.clone())))
        .and_then(
            |token: Option<String>,
             session_id: Option<String>,
             (jwks, redis_url): (_, Option<String>)| async move {
                let mut auth: Option<AuthSource> = None;

                // --- JWT path ---
                if let Some(ref t) = token {
                    match validate_token(t, &jwks) {
                        Ok(claims) => {
                            debug!(auth_method = "jwt", sub = %claims.sub, "Request authenticated");
                            auth = Some(AuthSource::Jwt(claims.sub));
                        }
                        Err(e) => {
                            warn!(auth_method = "jwt", error = %e, "JWT validation failed");
                        }
                    }
                }

                // --- Session fallback (guest) ---
                if auth.is_none() {
                    if let Some(ref sid) = session_id {
                        if let Some(ref rurl) = redis_url {
                            match security::session_validator::validate_session(sid, rurl).await {
                                Ok(user_id) => {
                                    debug!(auth_method = "session", "Request authenticated via session");
                                    auth = Some(AuthSource::Session(user_id));
                                }
                                Err(e) => {
                                    warn!(auth_method = "session", reason = %e, "Session validation failed");
                                }
                            }
                        } else {
                            warn!("X-Session-Id received but REDIS_URL is not configured");
                        }
                    }
                }

                // --- No valid credentials ---
                if auth.is_none() {
                    warn!(
                        has_jwt = token.is_some(),
                        has_session = session_id.is_some(),
                        "Request rejected: no valid authentication credentials"
                    );
                    return Err(warp::reject::custom(Unauthorized {}));
                }

                Ok::<Context, Rejection>(Context { jwks, redis_url, auth })
            },
        );

    let graphql_copy = warp::post()
        .and(warp::path("v2"))
        .and(juniper_warp::make_graphql_filter(schema(), context_filter.boxed()))
        .recover(handle_auth_rejection)
        .with(cors.clone())
        .with(warp::trace::request());

    let options_routes = warp::options().map(warp::reply).with(cors);

    let webhook_route = warp::post()
        .and(warp::path("webhook"))
        .and(warp::path::param::<String>()) // provider: e.g. "razorpay"
        .and(warp::header::optional::<String>("x-razorpay-signature"))
        .and(warp::body::bytes())
        .and_then(
            |provider: String, sig: Option<String>, body: warp::hyper::body::Bytes| async move {
                webhooks::handle_webhook(provider, sig, body)
                    .await
                    .map_err(|e| {
                        warn!("Webhook handler error: {:?}", e);
                        warp::reject::reject()
                    })
            },
        );

    // Bind address is configurable via GRAPHQL_LISTEN_ADDR (default: 0.0.0.0:8080)
    let listen_addr: std::net::SocketAddr = std::env::var("GRAPHQL_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("GRAPHQL_LISTEN_ADDR must be a valid socket address");

    info!(listen_addr = %listen_addr, "GraphQL service starting");

    warp::serve(
        load_balancer_health_check
            .or(graphql_copy)
            .or(webhook_route)
            .or(options_routes),
    )
    .run(listen_addr)
    .await
}

async fn handle_auth_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.is_not_found() {
        Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND))
    } else if err.find::<Unauthorized>().is_some() {
        Ok(reply::with_status("UNAUTHORIZED", StatusCode::UNAUTHORIZED))
    } else if let Some(_e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
    } else {
        warn!("Unhandled rejection: {:?}", err);
        Ok(reply::with_status(
            "INTERNAL_SERVER_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
