use juniper::{graphql_object, GraphQLInputObject};

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
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "New Cart Data")]
pub struct NewCart {
    pub user_id: String,
    pub product_id: String,
    pub quantity: String,
}
