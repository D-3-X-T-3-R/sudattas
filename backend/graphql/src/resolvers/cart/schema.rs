use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::product::schema::{Product, SearchProduct};

#[derive(Default, Debug, Clone)]
pub struct Cart {
    pub cart_id: String,
    pub user_id: String,
    pub product_id: String,
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

    async fn product_id(&self) -> &String {
        &self.product_id
    }

    async fn quantity(&self) -> &String {
        &self.quantity
    }

    async fn product_details(&self) -> FieldResult<Vec<Product>> {
        crate::resolvers::product::handlers::search_product(SearchProduct {
            product_id: Some(self.product_id.to_string()),
            name: None,
            description: None,
            starting_price: None,
            ending_price: None,
            stock_quantity: None,
            category_id: None,
            limit: None,
            offset: None,
        })
        .await
        .map_err(|e| e.into())
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Cart Data")]
pub struct NewCart {
    pub user_id: String,
    pub product_id: String,
    pub quantity: String,
    /// Guest cart: pass session_id from create_user when not logged in
    pub session_id: Option<String>,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct CartMutation {
    pub cart_id: String,
    pub user_id: String,
    pub product_id: String,
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
