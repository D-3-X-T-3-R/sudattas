use super::Context;
use crate::resolvers::{
    cart::{self, schema::Cart},
    product::{
        self,
        schema::{Product, SearchProduct},
    },
};
use juniper::FieldResult;

pub struct QueryRoot;

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    // Cart
    #[instrument(err, ret)]
    async fn get_cart_items(user_id: String) -> FieldResult<Vec<Cart>> {
        cart::handlers::get_cart_items(user_id)
            .await
            .map_err(|e| e.into())
    }

    // Product
    #[instrument(err, ret)]
    async fn search_product(search: SearchProduct) -> FieldResult<Vec<Product>> {
        product::handlers::search_product(search)
            .await
            .map_err(|e| e.into())
    }
}
