use super::Context;
use crate::resolvers::cart::{
    self,
    schema::{Cart, CartMutation, NewCart},
};
use juniper::FieldResult;

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    #[instrument(err, ret)]
    async fn add_cart_item(cart_item: NewCart) -> FieldResult<Vec<Cart>> {
        cart::handlers::add_cart_item(cart_item)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_cart_item(cart_id: String) -> FieldResult<Vec<Cart>> {
        cart::handlers::delete_cart_item(cart_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_cart_item(cart_item: CartMutation) -> FieldResult<Vec<Cart>> {
        cart::handlers::update_cart_item(cart_item)
            .await
            .map_err(|e| e.into())
    }
}
