use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Review {
    pub review_id: String,
    pub product_id: String,
    pub user_id: String,
    pub rating: i32,
    pub comment: String,
}

#[graphql_object]
#[graphql(description = "Product review")]
impl Review {
    async fn review_id(&self) -> &String {
        &self.review_id
    }
    async fn product_id(&self) -> &String {
        &self.product_id
    }
    async fn user_id(&self) -> &String {
        &self.user_id
    }
    async fn rating(&self) -> i32 {
        self.rating
    }
    async fn comment(&self) -> &String {
        &self.comment
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a new review")]
pub struct NewReview {
    pub product_id: String,
    pub user_id: String,
    pub rating: i32,
    pub comment: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update an existing review")]
pub struct ReviewMutation {
    pub review_id: String,
    pub product_id: Option<String>,
    pub user_id: Option<String>,
    pub rating: Option<i32>,
    pub comment: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search reviews")]
pub struct SearchReview {
    /// Filter by specific review ID; omit to return all
    pub review_id: Option<String>,
    /// Filter by product ID
    pub product_id: Option<String>,
    /// Filter by user ID
    pub user_id: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<String>,
    /// Number of results to skip for pagination
    pub offset: Option<String>,
}
