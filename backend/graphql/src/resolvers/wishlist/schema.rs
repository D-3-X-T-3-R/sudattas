use juniper::{graphql_object, FieldResult, GraphQLInputObject};

use crate::resolvers::product::schema::{Product, SearchProduct};

#[derive(Default, Debug, Clone)]
pub struct WishlistItem {
    pub wishlist_id: String,
    pub user_id: String,
    pub product_id: String,
    pub date_added: String,
}

#[graphql_object]
#[graphql(description = "WishlistItem Data")]
impl WishlistItem {
    async fn wishlist_id(&self) -> &String {
        &self.wishlist_id
    }

    async fn user_id(&self) -> &String {
        &self.user_id
    }

    async fn product_id(&self) -> &String {
        &self.product_id
    }

    async fn date_added(&self) -> &String {
        &self.date_added
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
#[graphql(description = "New WishlistItem Data")]
pub struct NewWishlistItem {
    pub user_id: String,
    pub product_id: String,
}

#[derive(Default, Debug, Clone, GraphQLInputObject)]
pub struct SearchWishlistItem {
    pub wishlist_id: Option<String>,
    pub user_id: String,
    pub product_id: Option<String>,
}

// #[derive(Default, Debug, Clone, GraphQLInputObject)]
// pub struct WishlistItemMutation {
//     pub wishlist_id: String,
//     pub user_id: String,
//     pub product_id: String,
//     pub date_added: String,
// }

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete WishlistItem Data")]
pub struct DeleteWishlistItem {
    pub user_id: String,
    pub wishlist_id: String,
}
