use dotenvy::dotenv;
use governor::{Quota, RateLimiter};
use graphql::graphql_handler;
use graphql::health;
use graphql::query_handler::{AuthSource, Context};
use graphql::schema;
use graphql::security::csrf;
use graphql::security::jwks_loader::load_jwks;
use graphql::security::jwt_validator::validate_token;
use graphql::security::session_validator;
use graphql::seo;
use graphql::webhooks;
use metrics_exporter_prometheus::PrometheusBuilder;
use reqwest::StatusCode;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, info, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use uuid::Uuid;
use warp::{http::Response, reply, Filter, Rejection, Reply};

#[derive(Debug)]
struct Unauthorized {}
impl warp::reject::Reject for Unauthorized {}

#[derive(Debug)]
struct RateLimited {}
impl warp::reject::Reject for RateLimited {}

/// P2 Security: CSRF — request with session auth from disallowed origin.
#[derive(Debug)]
struct CsrfRejected {}
impl warp::reject::Reject for CsrfRejected {}

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

    let jwks = load_jwks().await.expect("Failed to load JWKS");

    let redis_url = std::env::var("REDIS_URL").ok();

    // P2 Security: when set, session-authenticated POSTs must have Origin/Referer in this list.
    let allowed_origins: Vec<String> = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    let allowed_origins = if allowed_origins.is_empty() {
        None
    } else {
        Some(Arc::new(allowed_origins))
    };

    let rate_limit_per_minute: u32 = std::env::var("RATE_LIMIT_PER_MINUTE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let rate_limiter: Option<Arc<governor::DefaultKeyedRateLimiter<IpAddr>>> =
        if rate_limit_per_minute == 0 {
            None
        } else {
            let n = rate_limit_per_minute.max(1);
            let quota = Quota::per_minute(NonZeroU32::new(n).unwrap_or(NonZeroU32::MIN));
            Some(Arc::new(RateLimiter::keyed(quota)))
        };
    let rate_limit_filter = {
        let limiter = rate_limiter.clone();
        warp::addr::remote()
            .and(warp::any().map(move || limiter.clone()))
            .and_then(
                |addr: Option<std::net::SocketAddr>,
                 lim: Option<Arc<governor::DefaultKeyedRateLimiter<IpAddr>>>| async move {
                    if let (Some(addr), Some(ref l)) = (addr, lim) {
                        if l.check_key(&addr.ip()).is_err() {
                            return Err(warp::reject::custom(RateLimited {}));
                        }
                    }
                    Ok::<(), Rejection>(())
                },
            )
            .map(|_| ())
    };

    let webhook_limit_per_minute: u32 = std::env::var("RATE_LIMIT_WEBHOOK_PER_MINUTE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let webhook_rate_limiter: Option<Arc<governor::DefaultKeyedRateLimiter<IpAddr>>> =
        if webhook_limit_per_minute == 0 {
            None
        } else {
            let n = webhook_limit_per_minute.max(1);
            let quota = Quota::per_minute(NonZeroU32::new(n).unwrap_or(NonZeroU32::MIN));
            Some(Arc::new(RateLimiter::keyed(quota)))
        };
    let webhook_rate_limit_filter = {
        let limiter = webhook_rate_limiter.clone();
        warp::addr::remote()
            .and(warp::any().map(move || limiter.clone()))
            .and_then(
                |addr: Option<std::net::SocketAddr>,
                 lim: Option<Arc<governor::DefaultKeyedRateLimiter<IpAddr>>>| async move {
                    if let (Some(addr), Some(ref l)) = (addr, lim) {
                        if l.check_key(&addr.ip()).is_err() {
                            return Err(warp::reject::custom(RateLimited {}));
                        }
                    }
                    Ok::<(), Rejection>(())
                },
            )
            .map(|_| ())
    };

    let cors = warp::cors()
        .allow_any_origin()
        .allow_credentials(true)
        .allow_headers(vec![
            "content-type",
            "authorization",
            "x-session-id",
            "x-request-id",
            "idempotency-key",
        ])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    // Liveness: GET / — process is up (orchestrators use this for restart decisions).
    let load_balancer_health_check = warp::get().and(warp::path::end()).map(|| {
        Response::builder()
            .header("content-type", "text/plain")
            .body("OK")
    });

    // Readiness: GET /ready — dependencies (gRPC/DB, optional Redis) are up; use for traffic routing.
    let readiness_check = warp::get()
        .and(warp::path("ready"))
        .and(warp::path::end())
        .and_then(|| async move {
            match health::check_ready().await {
                Ok(()) => Ok::<_, std::convert::Infallible>(reply::with_status(
                    "OK".to_string(),
                    StatusCode::OK,
                )),
                Err(e) => {
                    warn!(error = %e, "Readiness check failed");
                    Ok::<_, std::convert::Infallible>(reply::with_status(
                        e,
                        StatusCode::SERVICE_UNAVAILABLE,
                    ))
                }
            }
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
    let allowed_origins_c = allowed_origins.clone();
    let context_filter = warp::header::optional::<String>("authorization")
        .and(warp::header::optional::<String>("x-session-id"))
        .and(warp::header::optional::<String>("origin"))
        .and(warp::header::optional::<String>("referer"))
        .and(warp::header::optional::<String>("x-request-id"))
        .and(warp::header::optional::<String>("idempotency-key"))
        .and(warp::any().map(move || (jwks_c.clone(), redis_url_c.clone(), allowed_origins_c.clone())))
        .and_then(
            |token: Option<String>,
             session_id: Option<String>,
             origin: Option<String>,
             referer: Option<String>,
             x_request_id: Option<String>,
             idempotency_key: Option<String>,
             (jwks, redis_url, allowed_origins): (_, Option<String>, Option<Arc<Vec<String>>>)| async move {
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
                            match session_validator::validate_session(sid, rurl).await {
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

                // --- P2 CSRF: session auth must come from an allowed origin when ALLOWED_ORIGINS is set ---
                if matches!(&auth, Some(AuthSource::Session(_))) {
                    if let Some(ref allowed) = allowed_origins {
                        let request_origin = origin
                            .as_ref()
                            .map(|o| o.trim().to_lowercase())
                            .or_else(|| {
                                referer.as_ref().and_then(|r| {
                                    csrf::parse_origin_from_referer(r).map(|s| s.to_lowercase())
                                })
                            });
                        let allowed = match request_origin {
                            Some(ref o) if !o.is_empty() => allowed.iter().any(|a| o == a),
                            _ => false,
                        };
                        if !allowed {
                            warn!("CSRF: session auth rejected — Origin/Referer missing or not in ALLOWED_ORIGINS");
                            return Err(warp::reject::custom(CsrfRejected {}));
                        }
                    }
                }

                let request_id =
                    x_request_id.or_else(|| Some(Uuid::new_v4().to_string()));
                Ok::<Context, Rejection>(Context {
                    jwks,
                    redis_url,
                    auth,
                    request_id,
                    idempotency_key,
                })
            },
        );

    let graphql_schema = Arc::new(schema());
    let graphql_route = warp::post()
        .and(warp::path("v2"))
        .and(warp::path::end())
        .and(context_filter.clone())
        .and(warp::body::bytes())
        .and_then({
            let schema = graphql_schema.clone();
            move |ctx: Context, body: warp::hyper::body::Bytes| {
                let schema = schema.clone();
                async move { graphql_handler::handle_graphql_request(ctx, body, schema).await }
            }
        })
        .recover(handle_auth_rejection)
        .with(cors.clone())
        .with(warp::trace::trace(
            |_| tracing::info_span!("request", request_id = %Uuid::new_v4()),
        ));
    let graphql_copy = rate_limit_filter
        .clone()
        .and(graphql_route)
        .map(|_, reply| reply);

    let options_routes = warp::options().map(warp::reply).with(cors);

    let webhook_route_inner = warp::post()
        .and(warp::path("webhook"))
        .and(warp::path::param::<String>()) // provider: e.g. "razorpay"
        .and(warp::header::optional::<String>("x-razorpay-signature"))
        .and(warp::header::optional::<String>("x-razorpay-event-id"))
        .and(warp::body::bytes())
        .and_then(
            |provider: String,
             sig: Option<String>,
             event_id: Option<String>,
             body: warp::hyper::body::Bytes| async move {
                webhooks::handle_webhook(provider, sig, event_id, body)
                    .await
                    .map_err(|e| {
                        warn!("Webhook handler error: {:?}", e);
                        warp::reject::reject()
                    })
            },
        );
    let webhook_route = webhook_rate_limit_filter
        .and(webhook_route_inner)
        .map(|_, reply| reply);

    // Bind address is configurable via GRAPHQL_LISTEN_ADDR (default: 0.0.0.0:8080)
    let listen_addr: std::net::SocketAddr = std::env::var("GRAPHQL_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("GRAPHQL_LISTEN_ADDR must be a valid socket address");

    let prom_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Prometheus metrics recorder");

    let metrics_route = warp::get()
        .and(warp::path("metrics"))
        .and(warp::path::end())
        .map(move || {
            let body = prom_handle.render();
            warp::reply::with_header(body, "content-type", "text/plain; charset=utf-8")
        });

    // P2 SEO: robots.txt and sitemap.xml (no auth)
    let robots_route = warp::get()
        .and(warp::path("robots.txt"))
        .and(warp::path::end())
        .map(|| {
            let body = seo::robots_txt();
            warp::reply::with_header(body, "content-type", "text/plain; charset=utf-8")
        });
    let sitemap_route = warp::get()
        .and(warp::path("sitemap.xml"))
        .and(warp::path::end())
        .and_then(|| async move {
            let reply: warp::reply::Response = match seo::sitemap_xml().await {
                Ok(xml) => {
                    warp::reply::with_header(xml, "content-type", "application/xml; charset=utf-8")
                        .into_response()
                }
                Err(_) => warp::reply::with_status(
                    "Internal error generating sitemap",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response(),
            };
            Ok::<_, Rejection>(reply)
        });

    info!(listen_addr = %listen_addr, "GraphQL service starting");

    // No catch-all route: unmatched paths reject. Top-level recover turns NotFound -> 404.
    let routes = load_balancer_health_check
        .or(readiness_check)
        .or(metrics_route)
        .or(robots_route)
        .or(sitemap_route)
        .or(graphql_copy)
        .or(webhook_route)
        .or(options_routes)
        .recover(handle_auth_rejection);

    warp::serve(routes).run(listen_addr).await
}

async fn handle_auth_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    // Check auth/rate-limit first: when graphql rejects (e.g. 401), we still try options() which
    // adds MethodNotAllowed; we must return 401 not 404 for POST /v2 with bad auth.
    if err.find::<Unauthorized>().is_some() {
        return Ok(reply::with_status("UNAUTHORIZED", StatusCode::UNAUTHORIZED));
    }
    if err.find::<CsrfRejected>().is_some() {
        return Ok(reply::with_status("FORBIDDEN", StatusCode::FORBIDDEN));
    }
    if err.find::<RateLimited>().is_some() {
        return Ok(reply::with_status(
            "TOO_MANY_REQUESTS",
            StatusCode::TOO_MANY_REQUESTS,
        ));
    }
    if let Some(_e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        return Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST));
    }
    if err.is_not_found() {
        return Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND));
    }
    if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        // No route matched (e.g. GET /unknown-path); last filter tried was options() -> 405.
        return Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND));
    }
    warn!("Unhandled rejection: {:?}", err);
    Ok(reply::with_status(
        "INTERNAL_SERVER_ERROR",
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
}
