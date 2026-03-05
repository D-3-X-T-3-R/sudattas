use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::product::schema::Product;

#[derive(Default, Debug, Clone)]
pub struct Cart {
    pub cart_id: String,
    pub user_id: String,
    pub variant_id: String,
    pub quantity: String,
}

#[graphql_object]
#[graphql(description = "Cart Data")]
impl Cart {
    async fn cart_id(&self) -> &String {
        &self.cart_id
    }

    async fn user_id(&self) -> &String {
        &self.user_id
    }

    async fn variant_id(&self) -> &String {
        &self.variant_id
    }

    async fn quantity(&self) -> &String {
        &self.quantity
    }

    async fn product_details(&self) -> FieldResult<Vec<Product>> {
        crate::resolvers::product::handlers::get_products_for_variant(&self.variant_id)
            .await
            .map_err(|e| e.into())
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Cart Data")]
pub struct NewCart {
    pub user_id: String,
    pub variant_id: String,
    pub quantity: String,
    /// Guest cart: pass session_id from create_user when not logged in
    pub session_id: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct CartMutation {
    pub cart_id: String,
    pub user_id: String,
    pub variant_id: String,
    pub quantity: String,
    pub session_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete Cart Data")]
pub struct DeleteCartItem {
    pub user_id: String,
    pub cart_id: Option<String>,
    /// Guest cart: pass session_id when not logged in
    pub session_id: Option<String>,
}
