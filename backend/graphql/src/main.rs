use dotenvy::dotenv;
use governor::{Quota, RateLimiter};
use graphql::graphql_handler;
use graphql::health;
use graphql::query_handler::admin_roles;
use graphql::query_handler::{AuthSource, Context};
use graphql::resolvers::error::Code as GqlCode;
use graphql::resolvers::invoices::handlers as invoice_handlers;
use graphql::schema;
use graphql::security::csrf;
use graphql::security::guest_session;
use graphql::security::jwks_loader::load_jwks;
use graphql::security::jwt_validator::validate_token;
use graphql::security::phone_otp;
use graphql::security::session_validator;
use graphql::seo;
mod startup_config;
use graphql::webhooks;
use metrics_exporter_prometheus::PrometheusBuilder;
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use tracing::{debug, info, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use uuid::Uuid;
use warp::http::StatusCode;
use warp::{http::Response, reply, Filter, Rejection, Reply};

#[derive(Deserialize)]
struct PhoneOtpRequestBody {
    phone: String,
    channel: Option<String>,
}

#[derive(Deserialize)]
struct PhoneOtpVerifyBody {
    phone: String,
    otp: String,
}

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

fn parse_first_forwarded_ip(raw: &str) -> Option<IpAddr> {
    for part in raw.split(',') {
        let candidate = part.trim();
        if candidate.is_empty() {
            continue;
        }
        if let Ok(ip) = candidate.parse::<IpAddr>() {
            return Some(ip);
        }
    }
    None
}

fn resolve_client_ip_for_rate_limit(
    remote: Option<SocketAddr>,
    x_forwarded_for: Option<&str>,
    x_real_ip: Option<&str>,
    trust_proxy_headers: bool,
) -> Option<IpAddr> {
    if trust_proxy_headers {
        if let Some(raw) = x_real_ip {
            if let Ok(ip) = raw.trim().parse::<IpAddr>() {
                return Some(ip);
            }
        }
        if let Some(raw) = x_forwarded_for {
            if let Some(ip) = parse_first_forwarded_ip(raw) {
                return Some(ip);
            }
        }
    }
    remote.map(|addr| addr.ip())
}

fn hash_rate_limit_key(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn resolve_graphql_rate_limit_key(
    remote: Option<SocketAddr>,
    x_forwarded_for: Option<&str>,
    x_real_ip: Option<&str>,
    session_id: Option<&str>,
    authorization: Option<&str>,
    trust_proxy_headers: bool,
) -> Option<String> {
    if let Some(ip) =
        resolve_client_ip_for_rate_limit(remote, x_forwarded_for, x_real_ip, trust_proxy_headers)
    {
        return Some(format!("ip:{ip}"));
    }
    if let Some(sid) = session_id {
        let trimmed = sid.trim();
        if !trimmed.is_empty() {
            return Some(format!("sid:{}", hash_rate_limit_key(trimmed)));
        }
    }
    if let Some(auth) = authorization {
        let trimmed = auth.trim();
        if !trimmed.is_empty() {
            return Some(format!("auth:{}", hash_rate_limit_key(trimmed)));
        }
    }
    None
}

fn status_from_gql_code(code: GqlCode) -> StatusCode {
    match code {
        GqlCode::InvalidArgument => StatusCode::BAD_REQUEST,
        GqlCode::NotFound => StatusCode::NOT_FOUND,
        GqlCode::PermissionDenied => StatusCode::FORBIDDEN,
        GqlCode::Unauthenticated => StatusCode::UNAUTHORIZED,
        GqlCode::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        GqlCode::FailedPrecondition => StatusCode::PRECONDITION_FAILED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

const CORS_ALLOWED_HEADERS: &[&str] = &[
    "content-type",
    "authorization",
    "x-session-id",
    "x-request-id",
    "idempotency-key",
    "x-client-action",
    "x-guest-session-id",
    "x-internal-auth",
    "x-customer-user-id",
];

const CORS_ALLOWED_METHODS: &[&str] = &["GET", "POST", "OPTIONS"];

fn build_cors(allowed_origins: Option<&[String]>) -> warp::filters::cors::Builder {
    let base = warp::cors()
        .allow_headers(CORS_ALLOWED_HEADERS.to_vec())
        .allow_methods(CORS_ALLOWED_METHODS.to_vec());

    match allowed_origins {
        Some(origins) if !origins.is_empty() => origins
            .iter()
            .fold(base.allow_credentials(true), |cors, origin| {
                cors.allow_origin(origin.as_str())
            }),
        _ => {
            warn!(
                "ALLOWED_ORIGINS is not configured; using permissive CORS only because strict startup validation is disabled"
            );
            base.allow_any_origin().allow_credentials(false)
        }
    }
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

    let startup = startup_config::StartupConfig::from_env()
        .unwrap_or_else(|e| panic!("invalid startup environment: {e}"));
    let jwks = load_jwks().await.expect("Failed to load JWKS");
    let redis_url = startup.redis_url.clone();
    let allowed_origins = startup.allowed_origins.clone().map(Arc::new);
    let rate_limit_per_minute = startup.rate_limit_per_minute;
    let trust_proxy_headers = startup.trust_proxy_headers;
    let rate_limiter: Option<Arc<governor::DefaultKeyedRateLimiter<String>>> =
        if rate_limit_per_minute == 0 {
            None
        } else {
            let n = rate_limit_per_minute.max(1);
            let quota = Quota::per_minute(NonZeroU32::new(n).unwrap_or(NonZeroU32::MIN));
            Some(Arc::new(RateLimiter::keyed(quota)))
        };
    let rate_limit_filter = {
        let limiter = rate_limiter.clone();
        let trust_proxy = trust_proxy_headers;
        warp::addr::remote()
            .and(warp::header::optional::<String>("x-forwarded-for"))
            .and(warp::header::optional::<String>("x-real-ip"))
            .and(warp::header::optional::<String>("x-session-id"))
            .and(warp::header::optional::<String>("authorization"))
            .and(warp::any().map(move || limiter.clone()))
            .and_then(
                move |addr: Option<SocketAddr>,
                      x_forwarded_for: Option<String>,
                      x_real_ip: Option<String>,
                      session_id: Option<String>,
                      authorization: Option<String>,
                      lim: Option<Arc<governor::DefaultKeyedRateLimiter<String>>>| async move {
                    if let Some(ref l) = lim {
                        let key = resolve_graphql_rate_limit_key(
                            addr,
                            x_forwarded_for.as_deref(),
                            x_real_ip.as_deref(),
                            session_id.as_deref(),
                            authorization.as_deref(),
                            trust_proxy,
                        );
                        if let Some(client_key) = key {
                            if l.check_key(&client_key).is_err() {
                                return Err(warp::reject::custom(RateLimited {}));
                            }
                        } else {
                            return Err(warp::reject::custom(RateLimited {}));
                        }
                    }
                    Ok::<(), Rejection>(())
                },
            )
            .map(|_| ())
    };

    let webhook_limit_per_minute = startup.webhook_rate_limit_per_minute;
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
        let trust_proxy = trust_proxy_headers;
        warp::addr::remote()
            .and(warp::header::optional::<String>("x-forwarded-for"))
            .and(warp::header::optional::<String>("x-real-ip"))
            .and(warp::any().map(move || limiter.clone()))
            .and_then(
                move |addr: Option<SocketAddr>,
                      x_forwarded_for: Option<String>,
                      x_real_ip: Option<String>,
                      lim: Option<Arc<governor::DefaultKeyedRateLimiter<IpAddr>>>| async move {
                    if let Some(ref l) = lim {
                        let ip = resolve_client_ip_for_rate_limit(
                            addr,
                            x_forwarded_for.as_deref(),
                            x_real_ip.as_deref(),
                            trust_proxy,
                        );
                        if let Some(client_ip) = ip {
                            if l.check_key(&client_ip).is_err() {
                                return Err(warp::reject::custom(RateLimited {}));
                            }
                        } else {
                            return Err(warp::reject::custom(RateLimited {}));
                        }
                    }
                    Ok::<(), Rejection>(())
                },
            )
            .map(|_| ())
    };
    info!(
        rate_limit_per_minute,
        webhook_limit_per_minute,
        trust_proxy_headers,
        enforce_webhook_secrets = startup.enforce_webhook_secrets,
        "Rate limiter configured"
    );

    let cors = build_cors(startup.allowed_origins.as_deref());

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
    //
    // Cross-layer boundary expectation:
    //   - Public storefront reads must use guest sessions (`X-Session-Id`).
    //   - Authenticated customer/admin flows must use JWT (`Authorization`).
    // Route families are documented in docs/CROSS_LAYER_CONTRACT.md.
    let jwks_c = jwks.clone();
    let redis_url_c = redis_url.clone();
    let allowed_origins_c = allowed_origins.clone();
    let context_filter = warp::header::optional::<String>("authorization")
        .and(warp::header::optional::<String>("x-session-id"))
        .and(warp::header::optional::<String>("x-internal-auth"))
        .and(warp::header::optional::<String>("x-customer-user-id"))
        .and(warp::header::optional::<String>("origin"))
        .and(warp::header::optional::<String>("referer"))
        .and(warp::header::optional::<String>("x-request-id"))
        .and(warp::header::optional::<String>("idempotency-key"))
        .and(warp::header::optional::<String>("x-client-action"))
        .and(warp::header::optional::<String>("x-guest-session-id"))
        .and(warp::any().map(move || (jwks_c.clone(), redis_url_c.clone(), allowed_origins_c.clone())))
        .and_then(
            |token: Option<String>,
             session_id: Option<String>,
             x_internal_auth: Option<String>,
             x_customer_user_id: Option<String>,
             origin: Option<String>,
             referer: Option<String>,
             x_request_id: Option<String>,
             idempotency_key: Option<String>,
             x_client_action: Option<String>,
             x_guest_session_id: Option<String>,
             (jwks, redis_url, allowed_origins): (_, Option<String>, Option<Arc<Vec<String>>>)| async move {
                let mut auth: Option<AuthSource> = None;
                let mut jwt_subject: Option<String> = None;
                let mut admin_authorized: Option<bool> = None;
                let mut admin_resolution_source: Option<String> = None;
                let request_id = x_request_id.or_else(|| Some(Uuid::new_v4().to_string()));

                // --- JWT path ---
                if let Some(ref t) = token {
                    match validate_token(t, &jwks) {
                        Ok(claims) => {
                            jwt_subject = Some(claims.sub.clone());
                            let auth_user_id = claims.user_id.unwrap_or_else(|| claims.sub.clone());
                            debug!(auth_method = "jwt", sub = %claims.sub, auth_user_id = %auth_user_id, "Request authenticated");
                            auth = Some(AuthSource::Jwt(auth_user_id));
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

                // --- Admin role resolution (JWT only): DB/cache lookup by JWT sub ---
                if matches!(auth, Some(AuthSource::Jwt(_))) {
                    if let Some(sub) = jwt_subject.as_deref() {
                        match admin_roles::resolve_admin_from_db(sub, request_id.as_deref()).await {
                            Ok(resolution) => {
                                admin_authorized = Some(resolution.is_admin);
                                admin_resolution_source = Some(resolution.source.to_string());
                                graphql::metrics::record_admin_role_resolution_total(
                                    resolution.source,
                                    "success",
                                );
                            }
                            Err(err) => {
                                warn!(error = %err, "Admin role lookup failed");
                                graphql::metrics::record_admin_role_resolution_total("db", "failure");
                            }
                        }
                    }
                }
                if matches!(auth, Some(AuthSource::Jwt(_))) && admin_resolution_source.is_none() {
                    admin_resolution_source = Some("env_fallback".to_string());
                }

                // --- Trusted internal auth (server-side frontend proxy) ---
                if auth.is_none() {
                    let configured_secret = std::env::var("INTERNAL_API_SECRET")
                        .ok()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    let provided_secret = x_internal_auth
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty());

                    if let (Some(expected), Some(got)) = (configured_secret.as_deref(), provided_secret)
                    {
                        if expected == got {
                            if let Some(uid) = x_customer_user_id
                                .as_deref()
                                .map(str::trim)
                                .filter(|s| !s.is_empty())
                            {
                                if uid.chars().all(|c| c.is_ascii_digit()) {
                                    debug!(
                                        auth_method = "internal_customer",
                                        user_id = %uid,
                                        "Request authenticated via internal customer auth"
                                    );
                                    auth = Some(AuthSource::InternalCustomer(uid.to_string()));
                                } else {
                                    warn!("Internal auth rejected: non-numeric x-customer-user-id");
                                }
                            } else {
                                debug!(
                                    auth_method = "internal_service",
                                    "Request authenticated via internal service auth"
                                );
                                auth = Some(AuthSource::InternalService);
                            }
                        } else {
                            warn!("Internal auth rejected: secret mismatch");
                        }
                    }
                }

                // --- No valid credentials ---
                if auth.is_none() {
                    warn!(
                        has_jwt = token.is_some(),
                        has_session = session_id.is_some(),
                        has_internal = x_internal_auth.is_some(),
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

                let resolved_guest_session_id = session_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .or_else(|| {
                        x_guest_session_id
                            .as_deref()
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                    });

                Ok::<Context, Rejection>(Context {
                    jwks,
                    redis_url,
                    auth,
                    request_id,
                    idempotency_key,
                    client_action: x_client_action,
                    guest_session_id: resolved_guest_session_id,
                    jwt_subject,
                    admin_authorized,
                    admin_resolution_source,
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
        .with(cors.clone())
        .with(warp::trace::trace(
            |_| tracing::info_span!("request", request_id = %Uuid::new_v4()),
        ));
    let graphql_copy = rate_limit_filter
        .clone()
        .and(graphql_route)
        .map(|_, reply| reply);

    let invoice_download_route = warp::get()
        .and(warp::path("invoices"))
        .and(warp::path::param::<String>())
        .and(warp::path("download"))
        .and(warp::path::end())
        .and(context_filter.clone())
        .and_then(|invoice_number: String, ctx: Context| async move {
            match invoice_handlers::download_invoice_pdf(&ctx, invoice_number.as_str()).await {
                Ok(payload) => {
                    let reply = reply::with_header(
                        reply::with_header(
                            reply::with_header(
                                reply::with_status(payload.pdf_bytes, StatusCode::OK),
                                "Content-Type",
                                payload.content_type,
                            ),
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", payload.file_name),
                        ),
                        "Cache-Control",
                        "private, no-store",
                    );
                    Ok::<_, Rejection>(reply.into_response())
                }
                Err(err) => Ok::<_, Rejection>(
                    reply::with_status(err.message, status_from_gql_code(err.code)).into_response(),
                ),
            }
        });

    let options_routes = warp::options().map(warp::reply).with(cors.clone());

    let shiprocket_webhook_route_inner = warp::post()
        .and(warp::path("blastoff"))
        .and(warp::path("parcelupdate"))
        .and(warp::path::end())
        .and(warp::header::optional::<String>("x-razorpay-signature"))
        .and(warp::header::optional::<String>("x-razorpay-event-id"))
        .and(warp::header::optional::<String>("x-shiprocket-token"))
        .and(warp::header::optional::<String>("x-api-key"))
        .and(warp::body::bytes())
        .and_then(
            |sig: Option<String>,
             event_id: Option<String>,
             shiprocket_token: Option<String>,
             shiprocket_api_key: Option<String>,
             body: warp::hyper::body::Bytes| async move {
                let resolved_shiprocket_token = shiprocket_token.or(shiprocket_api_key);
                webhooks::handle_webhook(
                    "shiprocket".to_string(),
                    sig,
                    event_id,
                    resolved_shiprocket_token,
                    body,
                )
                .await
                .map_err(|e| {
                    warn!("Webhook handler error: {:?}", e);
                    warp::reject::reject()
                })
            },
        );
    let razorpay_webhook_route_inner = warp::post()
        .and(warp::path("wheresthemoney"))
        .and(warp::path("razorpay"))
        .and(warp::path::end())
        .and(warp::header::optional::<String>("x-razorpay-signature"))
        .and(warp::header::optional::<String>("x-razorpay-event-id"))
        .and(warp::header::optional::<String>("x-shiprocket-token"))
        .and(warp::body::bytes())
        .and_then(
            |sig: Option<String>,
             event_id: Option<String>,
             shiprocket_token: Option<String>,
             body: warp::hyper::body::Bytes| async move {
                webhooks::handle_webhook(
                    "razorpay".to_string(),
                    sig,
                    event_id,
                    shiprocket_token,
                    body,
                )
                .await
                .map_err(|e| {
                    warn!("Webhook handler error: {:?}", e);
                    warp::reject::reject()
                })
            },
        );
    let webhook_route = webhook_rate_limit_filter
        .and(shiprocket_webhook_route_inner.or(razorpay_webhook_route_inner))
        .map(|_, reply| reply);

    // Bind address is configurable via GRAPHQL_LISTEN_ADDR (default: 0.0.0.0:8080)
    let listen_addr = startup.listen_addr;

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

    // Guest session: POST /session/guest — create session in Redis, return { session_id } (no auth; requires REDIS_URL)
    // Always responds with JSON: 200 { session_id }, 503 { error }, or 405 { error } for GET.
    let redis_url_guest = redis_url.clone();
    let guest_session_get = warp::get()
        .and(warp::path("session"))
        .and(warp::path("guest"))
        .and(warp::path::end())
        .map(|| {
            let body =
                serde_json::json!({ "message": "Use POST to create a guest session" }).to_string();
            warp::reply::with_header(
                warp::reply::with_status(body, StatusCode::OK),
                "content-type",
                "application/json",
            )
        });
    let guest_session_route = warp::post()
        .and(warp::path("session"))
        .and(warp::path("guest"))
        .and(warp::path::end())
        .and(warp::any().map(move || redis_url_guest.clone()))
        .and_then(|redis_url_opt: Option<String>| async move {
            let (status, body) = match redis_url_opt {
                Some(rurl) => match guest_session::create_guest_session(&rurl).await {
                    Ok(session_id) => (
                        StatusCode::OK,
                        serde_json::json!({ "session_id": session_id }).to_string(),
                    ),
                    Err(e) => {
                        warn!(error = %e, "Guest session create failed");
                        (
                            StatusCode::SERVICE_UNAVAILABLE,
                            serde_json::json!({ "error": e }).to_string(),
                        )
                    }
                },
                None => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    serde_json::json!({ "error": "Guest sessions disabled (REDIS_URL not set)" })
                        .to_string(),
                ),
            };
            Ok::<_, Rejection>(
                warp::reply::with_header(
                    warp::reply::with_status(body, status),
                    "content-type",
                    "application/json",
                )
                .into_response(),
            )
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

    // Public auth helper routes (no session/JWT required):
    // POST /auth/phone-otp/request { phone }
    // POST /auth/phone-otp/verify  { phone, otp }
    let otp_request_inner = warp::post()
        .and(warp::path("auth"))
        .and(warp::path("phone-otp"))
        .and(warp::path("request"))
        .and(warp::path::end())
        .and(warp::body::json::<PhoneOtpRequestBody>())
        .and_then(|body: PhoneOtpRequestBody| async move {
            let (status, payload) =
                match phone_otp::request_sms_otp(&body.phone, body.channel.as_deref()).await {
                    Ok(()) => (
                        StatusCode::OK,
                        serde_json::json!({ "ok": true }).to_string(),
                    ),
                    Err(code) => match code.as_str() {
                        "INVALID_PHONE" => (
                            StatusCode::BAD_REQUEST,
                            serde_json::json!({ "ok": false, "error": code }).to_string(),
                        ),
                        "INVALID_CHANNEL" => (
                            StatusCode::BAD_REQUEST,
                            serde_json::json!({ "ok": false, "error": code }).to_string(),
                        ),
                        "OTP_NOT_CONFIGURED" => (
                            StatusCode::SERVICE_UNAVAILABLE,
                            serde_json::json!({ "ok": false, "error": code }).to_string(),
                        ),
                        _ => (
                            StatusCode::BAD_GATEWAY,
                            serde_json::json!({ "ok": false, "error": code }).to_string(),
                        ),
                    },
                };
            Ok::<_, Rejection>(
                warp::reply::with_header(
                    warp::reply::with_status(payload, status),
                    "content-type",
                    "application/json",
                )
                .into_response(),
            )
        });
    let otp_request_route = rate_limit_filter
        .clone()
        .and(otp_request_inner)
        .map(|_, reply| reply);

    let otp_verify_inner = warp::post()
        .and(warp::path("auth"))
        .and(warp::path("phone-otp"))
        .and(warp::path("verify"))
        .and(warp::path::end())
        .and(warp::body::json::<PhoneOtpVerifyBody>())
        .and_then(|body: PhoneOtpVerifyBody| async move {
            let (status, payload) = match phone_otp::verify_sms_otp(&body.phone, &body.otp).await {
                Ok(approved) => (
                    StatusCode::OK,
                    serde_json::json!({ "ok": approved, "approved": approved }).to_string(),
                ),
                Err(code) => match code.as_str() {
                    "INVALID_PHONE" | "INVALID_OTP" => (
                        StatusCode::BAD_REQUEST,
                        serde_json::json!({ "ok": false, "error": code }).to_string(),
                    ),
                    "OTP_NOT_CONFIGURED" => (
                        StatusCode::SERVICE_UNAVAILABLE,
                        serde_json::json!({ "ok": false, "error": code }).to_string(),
                    ),
                    _ => (
                        StatusCode::BAD_GATEWAY,
                        serde_json::json!({ "ok": false, "error": code }).to_string(),
                    ),
                },
            };
            Ok::<_, Rejection>(
                warp::reply::with_header(
                    warp::reply::with_status(payload, status),
                    "content-type",
                    "application/json",
                )
                .into_response(),
            )
        });
    let otp_verify_route = rate_limit_filter
        .clone()
        .and(otp_verify_inner)
        .map(|_, reply| reply);

    info!(listen_addr = %listen_addr, "GraphQL service starting");

    // No catch-all route: unmatched paths reject. Top-level recover turns NotFound -> 404.
    let routes = load_balancer_health_check
        .or(readiness_check)
        .or(metrics_route)
        .or(guest_session_get.with(cors.clone()))
        .or(guest_session_route.with(cors.clone()))
        .or(otp_request_route.with(cors.clone()))
        .or(otp_verify_route.with(cors.clone()))
        .or(robots_route)
        .or(sitemap_route)
        .or(invoice_download_route.with(cors.clone()))
        .or(graphql_copy)
        .or(webhook_route)
        .or(options_routes)
        .recover(handle_auth_rejection);

    warp::serve(routes).run(listen_addr).await
}

async fn handle_auth_rejection(
    err: Rejection,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    // Check auth/rate-limit first: when graphql rejects (e.g. 401), we still try options() which
    // adds MethodNotAllowed; we must return 401 not 404 for POST /v2 with bad auth.
    if err.find::<Unauthorized>().is_some() {
        graphql::metrics::record_auth_rejection_total("unauthorized");
        return Ok(reply::with_status("UNAUTHORIZED", StatusCode::UNAUTHORIZED).into_response());
    }
    if err.find::<CsrfRejected>().is_some() {
        graphql::metrics::record_auth_rejection_total("csrf");
        return Ok(reply::with_status("FORBIDDEN", StatusCode::FORBIDDEN).into_response());
    }
    if err.find::<RateLimited>().is_some() {
        graphql::metrics::record_auth_rejection_total("rate_limited");
        return Ok(reply::with_header(
            reply::with_status("TOO_MANY_REQUESTS", StatusCode::TOO_MANY_REQUESTS),
            "retry-after",
            "1",
        )
        .into_response());
    }
    if let Some(_e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        return Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST).into_response());
    }
    if err.is_not_found() {
        return Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND).into_response());
    }
    if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        // No route matched (e.g. GET /unknown-path); last filter tried was options() -> 405.
        return Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND).into_response());
    }
    warn!("Unhandled rejection: {:?}", err);
    Ok(
        reply::with_status("INTERNAL_SERVER_ERROR", StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    )
}

#[cfg(test)]
mod cors_tests {
    use super::*;

    #[tokio::test]
    async fn cors_allows_configured_origin_with_credentials() {
        let origins = vec!["https://app.example.com".to_string()];
        let route = warp::any()
            .map(warp::reply)
            .with(build_cors(Some(&origins)));
        let response = warp::test::request()
            .method("OPTIONS")
            .header("origin", "https://app.example.com")
            .header("access-control-request-method", "POST")
            .reply(&route)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()["access-control-allow-origin"],
            "https://app.example.com"
        );
        assert_eq!(
            response.headers()["access-control-allow-credentials"],
            "true"
        );
    }

    #[tokio::test]
    async fn cors_rejects_unconfigured_origin() {
        let origins = vec!["https://app.example.com".to_string()];
        let route = warp::any()
            .map(warp::reply)
            .with(build_cors(Some(&origins)));
        let response = warp::test::request()
            .method("OPTIONS")
            .header("origin", "https://evil.example")
            .header("access-control-request-method", "POST")
            .reply(&route)
            .await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(response
            .headers()
            .get("access-control-allow-origin")
            .is_none());
    }

    #[tokio::test]
    async fn permissive_non_strict_fallback_does_not_enable_credentials() {
        let route = warp::any().map(warp::reply).with(build_cors(None));
        let response = warp::test::request()
            .method("OPTIONS")
            .header("origin", "https://random.example")
            .header("access-control-request-method", "POST")
            .reply(&route)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()["access-control-allow-origin"],
            "https://random.example"
        );
        assert!(response
            .headers()
            .get("access-control-allow-credentials")
            .is_none());
    }
}
