// use reqwest::StatusCode;
// use tracing::{debug, warn};
// use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
// use warp::{http::Response, reject::MissingHeader, reply, Filter, Rejection, Reply};
// use dotenv::dotenv;
// use std::env;
// use juniper::{EmptySubscription, RootNode};
// use resolvers::Context;

// use crate::security::jwt_validator::{validate_token, Claims};

// #[derive(Debug)]
// struct Unauthorized {}
// impl warp::reject::Reject for Unauthorized {}

// mod resolvers;
// mod security;

// type Schema =
//     RootNode<'static, resolvers::QueryRoot, resolvers::MutationRoot, EmptySubscription<Context>>;
// fn schema() -> Schema {
//     RootNode::new(
//         resolvers::QueryRoot {},
//         resolvers::MutationRoot {},
//         EmptySubscription::<Context>::new(),
//     )
// }

// struct ContextFilter(resolvers::Context, warp::http::Response<Vec<u8>>);
// impl Reply for ContextFilter {
//     fn into_response(self) -> warp::http::Response<warp::hyper::Body> {
//         self.1.into_response()
//     }
// }

// #[tokio::main]
// async fn main() {
//     dotenv().ok();

//     tracing_subscriber::fmt()
//         // Display log level
//         .with_level(true)
//         // Get filter from RUST_LOG env var
//         .with_env_filter(EnvFilter::from_default_env())
//         // Display source code file paths
//         .with_file(true)
//         // Display source code line numbers
//         .with_line_number(true)
//         // Display the thread ID an event was recorded on
//         .with_thread_ids(true)
//         // Don't display the event's target (module path)
//         .with_target(false)
//         .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
//         // Build the subscriber
//         .init();

//     let jwks = security::jwks_loader::load_jwks()
//         .await
//         .expect("Failed to load JWKS");

//     let context = Context { jwks: jwks.clone() };

//     let cors = warp::cors()
//         .allow_any_origin()
//         .allow_credentials(true)
//         .allow_headers(vec!["content-type", "authorization"])
//         .allow_methods(vec!["GET", "POST", "OPTIONS"]);

//     let state = warp::any().map(move || context.clone());

//     let context_extractor = warp::any().and(warp::header::<String>("authorization").map(
//         move |token: String| match validate_token(&token, &jwks.clone()) {
//             Ok(claims) => {
//                 debug!("Validated token with claims: {:#?}", claims);
//                 Ok(claims)
//             }
//             Err(e) => {
//                 warn!("Token failed validation: {e:#?}");
//                 Err(warp::reject::custom(Unauthorized {}))
//             }
//         },
//     ));

//     let load_balancer_health_check = warp::get().and(warp::path::end()).map(|| {
//         Response::builder()
//             .header("content-type", "text/plain")
//             .body("OK")
//     });

//     let graphql = warp::post()
//         .and(warp::path("v1"))
//         .and(
//             context_extractor
//                 .and_then(|ctx| async move {
//                     match ctx {
//                         Ok(claims) => Ok(claims),
//                         Err(e) => {
//                             warn!("Rejecting request: {e:#?}");
//                             Err::<Claims, Rejection>(warp::reject::custom(Unauthorized {}))
//                         }
//                     }
//                 })
//                 .and(juniper_warp::make_graphql_filter(schema(), state.boxed()))
//                 .map(|_claims, response| response),
//         )
//         .recover(handle_auth_rejection)
//         .with(cors.clone())
//         .with(warp::trace::request());

//     let options_routes = warp::options().map(warp::reply).with(cors);

//     tracing::info!("Listening on 0.0.0.0:8080");

//     warp::serve(load_balancer_health_check.or(graphql).or(options_routes))
//         .run(([0, 0, 0, 0], 8080))
//         .await
// }

// // Custom rejection handler that maps rejections into responses.
// async fn handle_auth_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
//     if err.is_not_found() {
//         Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND))
//     } else if let Some(_e) = err.find::<warp::filters::body::BodyDeserializeError>() {
//         Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
//     } else if let Some(_e) = err.find::<MissingHeader>() {
//         Ok(reply::with_status(
//             "Missing header",
//             StatusCode::BAD_REQUEST,
//         ))
//     } else if let Some(_e) = err.find::<Unauthorized>() {
//         Ok(reply::with_status("Unauthorized", StatusCode::UNAUTHORIZED))
//     } else {
//         eprintln!("unhandled rejection: {:?}", err);
//         Ok(reply::with_status(
//             "INTERNAL_SERVER_ERROR",
//             StatusCode::INTERNAL_SERVER_ERROR,
//         ))
//     }
// }

use reqwest::StatusCode;
use tracing::{info, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use warp::{http::Response, Filter, Rejection, Reply};
use dotenv::dotenv;
use juniper::{EmptySubscription, RootNode};
use resolvers::Context;

mod resolvers;
mod security;

type Schema = RootNode<'static, resolvers::QueryRoot, resolvers::MutationRoot, EmptySubscription<Context>>;

fn schema() -> Schema {
    RootNode::new(
        resolvers::QueryRoot {},
        resolvers::MutationRoot {},
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
        .init();

    let context = Context {}; // Assuming Context does not require jwks anymore.

    let cors = warp::cors()
        .allow_any_origin()
        .allow_credentials(true)
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    let state = warp::any().map(move || context.clone());

    let load_balancer_health_check = warp::get().and(warp::path::end()).map(|| {
        Response::builder()
            .header("content-type", "text/plain")
            .body("OK")
    });

    let graphql = warp::post()
        .and(warp::path("v1"))
        .and(juniper_warp::make_graphql_filter(schema(), state.boxed()))
        .with(cors.clone())
        .with(warp::trace::request());

    let options_routes = warp::options().map(warp::reply).with(cors);

    info!("Listening on 0.0.0.0:8080");

    warp::serve(load_balancer_health_check.or(graphql).or(options_routes))
        .run(([0, 0, 0, 0], 8080))
        .await
}

// Custom rejection handler that maps rejections into responses.
// async fn handle_auth_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
//     if err.is_not_found() {
//         Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND))
//     } else if let Some(_e) = err.find::<warp::filters::body::BodyDeserializeError>() {
//         Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
//     } else {
//         warn!("Unhandled rejection: {:?}", err);
//         Ok(reply::with_status(
//             "INTERNAL_SERVER_ERROR",
//             StatusCode::INTERNAL_SERVER_ERROR,
//         ))
//     }
// }
