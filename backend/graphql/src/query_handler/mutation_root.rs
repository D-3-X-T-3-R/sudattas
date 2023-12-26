use super::Context;
use crate::resolvers::{
    cart::{
        self,
        schema::{Cart, CartMutation, NewCart},
    },
    category::{
        self,
        schema::{Category, CategoryMutation, NewCategory},
    },
    product::{
        self,
        schema::{NewProduct, Product, ProductMutation},
    },
};
use juniper::FieldResult;

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    // Cart
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

    // Product
    #[instrument(err, ret)]
    async fn create_product(product: NewProduct) -> FieldResult<Vec<Product>> {
        product::handlers::create_product(product)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_product(product_id: String) -> FieldResult<Vec<Product>> {
        product::handlers::delete_product(product_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_product(product: ProductMutation) -> FieldResult<Vec<Product>> {
        product::handlers::update_product(product)
            .await
            .map_err(|e| e.into())
    }

    // Category
    #[instrument(err, ret)]
    async fn create_category(category: NewCategory) -> FieldResult<Vec<Category>> {
        category::handlers::create_category(category)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_category(category_id: String) -> FieldResult<Vec<Category>> {
        category::handlers::delete_category(category_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_category(category: CategoryMutation) -> FieldResult<Vec<Category>> {
        category::handlers::update_category(category)
            .await
            .map_err(|e| e.into())
    }
}
