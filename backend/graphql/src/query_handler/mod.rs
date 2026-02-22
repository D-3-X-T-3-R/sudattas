use crate::security::jwks_loader::JWKSet;
use warp::Reply;

pub mod mutation_root;
pub mod query_root;

#[derive(Clone, Debug)]
pub struct Context {
    pub jwks: JWKSet,
}

impl Context {
    /// JWKS used for JWT validation (read by auth filter).
    pub fn jwks(&self) -> &JWKSet {
        &self.jwks
    }
}

impl Reply for Context {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::Response::new("foo".into())
    }
}
