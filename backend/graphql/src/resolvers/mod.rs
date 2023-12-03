use crate::{
    resolvers::cart::schema::{Cart, NewCart},
    security::jwks_loader::JWKSet,
};
use juniper::FieldResult;
use warp::Reply;

pub mod cart;
pub mod error;
pub mod utils;

pub struct QueryRoot;

#[derive(Clone, Debug)]
pub struct Context {
    pub jwks: JWKSet,
}

impl Reply for Context {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::Response::new("foo".into())
    }
}

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    #[instrument(err, ret)]
    async fn add_cart_item(cart_item: NewCart) -> FieldResult<Vec<Cart>> {
        cart::handlers::add_cart_item(cart_item)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn get_cart_items(user_id: String) -> FieldResult<Vec<Cart>> {
        cart::handlers::get_cart_items(user_id)
            .await
            .map_err(|e| e.into())
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    fn test() -> FieldResult<String> {
        Ok("ok".to_string())
    }
}
