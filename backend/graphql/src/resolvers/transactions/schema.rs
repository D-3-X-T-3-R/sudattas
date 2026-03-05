use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct Transaction {
    pub transaction_id: String,
    pub user_id: String,
    pub amount_paise: String,
    pub transaction_date: String,
    pub r#type: String,
}

#[graphql_object]
#[graphql(description = "Transaction")]
impl Transaction {
    async fn transaction_id(&self) -> &String {
        &self.transaction_id
    }

    async fn user_id(&self) -> &String {
        &self.user_id
    }

    async fn amount_paise(&self) -> &String {
        &self.amount_paise
    }

    async fn transaction_date(&self) -> &String {
        &self.transaction_date
    }

    async fn r#type(&self) -> &String {
        &self.r#type
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a transaction")]
pub struct NewTransaction {
    pub user_id: String,
    pub amount_paise: String,
    pub r#type: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search a transaction by ID")]
pub struct SearchTransactionInput {
    pub transaction_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a transaction")]
pub struct TransactionMutation {
    pub transaction_id: String,
    pub user_id: Option<String>,
    pub amount_paise: Option<String>,
    pub r#type: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a transaction")]
pub struct DeleteTransactionInput {
    pub transaction_id: String,
}
