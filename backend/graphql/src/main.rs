use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Schema,
};
use async_graphql_warp::graphql;
use warp::{http::Response as HttpResponse, Filter};

// Assume QueryRoot and MutationRoot are defined in your schema module
use schema::{create_schema, QueryRoot};

mod schema;
#[tokio::main]
async fn main() {
    let schema = create_schema();

    let graphql_route = warp::path("graphql").and(graphql(schema.clone()).and_then(
        |(schema, request): (
            Schema<QueryRoot, EmptyMutation, EmptySubscription>,
            async_graphql::Request,
        )| async move {
            let response = schema.execute(request).await;
            Ok::<_, warp::Rejection>(warp::reply::json(&response))
        },
    ));

    let playground_route = warp::path("playground").and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
    });

    warp::serve(graphql_route.or(playground_route))
        .run(([0, 0, 0, 0], 5000))
        .await;
}
