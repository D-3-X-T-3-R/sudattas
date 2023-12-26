use crate::security::jwks_loader::JWKSet;
use warp::Reply;

pub mod mutation_root;
pub mod query_root;

#[derive(Clone, Debug)]
pub struct Context {
    pub jwks: JWKSet,
}

impl Reply for Context {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::Response::new("foo".into())
    }
}
