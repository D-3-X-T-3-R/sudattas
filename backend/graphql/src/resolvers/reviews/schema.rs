use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Review {
    pub review_id: String,
    pub product_id: String,
    pub user_id: String,
    pub rating: i32,
    pub comment: String,
    /// "pending" | "approved" | "rejected"
    pub review_status: String,
    pub is_verified_purchase: bool,
    /// RFC3339; empty if unset.
    pub created_at: String,
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
    async fn review_status(&self) -> &String {
        &self.review_status
    }
    async fn is_verified_purchase(&self) -> bool {
        self.is_verified_purchase
    }
    async fn created_at(&self) -> &String {
        &self.created_at
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
    /// "pending" | "approved" | "rejected"; omit for all statuses
    pub status_filter: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Admin: update review status (approve/reject)")]
pub struct AdminUpdateReviewStatusInput {
    pub review_id: String,
    /// \"approved\" | \"rejected\"
    pub status: String,
}

/// Server-computed rating aggregate for a product. `average_rating` is CEIL(AVG(Rating)) — e.g.
/// a raw average of 3.2 or 3.8 both come back as 4 — and 0 when the product has no ratings yet.
#[derive(Default, Debug, Clone)]
pub struct ProductRatingSummary {
    pub product_id: String,
    pub average_rating: i32,
    pub rating_count: i32,
}

#[graphql_object]
#[graphql(description = "Server-computed star rating aggregate for a product (ceil-rounded)")]
impl ProductRatingSummary {
    async fn product_id(&self) -> &String {
        &self.product_id
    }
    async fn average_rating(&self) -> i32 {
        self.average_rating
    }
    async fn rating_count(&self) -> i32 {
        self.rating_count
    }
}
