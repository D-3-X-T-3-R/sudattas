use super::Context;
use crate::resolvers::cart::{self, schema::Cart};
use juniper::FieldResult;

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    #[instrument(err, ret)]
    async fn get_cart_items(user_id: String) -> FieldResult<Vec<Cart>> {
        cart::handlers::get_cart_items(user_id)
            .await
            .map_err(|e| e.into())
    }
}
