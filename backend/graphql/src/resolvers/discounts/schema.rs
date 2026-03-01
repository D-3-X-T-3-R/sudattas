use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Discount {
    pub discount_id: String,
    pub product_id: String,
    /// Discount in basis points (100 = 1%, 1050 = 10.5%)
    pub discount_percentage_basis_points: i32,
    pub start_date: String,
    pub end_date: String,
}

#[graphql_object]
#[graphql(description = "Product discount")]
impl Discount {
    async fn discount_id(&self) -> &String {
        &self.discount_id
    }
    async fn product_id(&self) -> &String {
        &self.product_id
    }
    /// Discount in basis points (100 = 1%, 1050 = 10.5%)
    async fn discount_percentage_basis_points(&self) -> i32 {
        self.discount_percentage_basis_points
    }
    async fn start_date(&self) -> &String {
        &self.start_date
    }
    async fn end_date(&self) -> &String {
        &self.end_date
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a discount")]
pub struct NewDiscount {
    pub product_id: String,
    /// Basis points (100 = 1%, 1050 = 10.5%)
    pub discount_percentage_basis_points: i32,
    pub start_date: String,
    pub end_date: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a discount")]
pub struct DiscountMutation {
    pub discount_id: String,
    pub product_id: Option<String>,
    pub discount_percentage_basis_points: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search discounts")]
pub struct SearchDiscount {
    pub discount_id: Option<String>,
}
