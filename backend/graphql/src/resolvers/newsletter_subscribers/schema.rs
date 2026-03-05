use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct NewsletterSubscriber {
    pub subscriber_id: String,
    pub email: String,
    pub subscription_date: String,
}

#[graphql_object]
#[graphql(description = "Newsletter subscriber")]
impl NewsletterSubscriber {
    async fn subscriber_id(&self) -> &String {
        &self.subscriber_id
    }

    async fn email(&self) -> &String {
        &self.email
    }

    async fn subscription_date(&self) -> &String {
        &self.subscription_date
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Create a newsletter subscriber")]
pub struct NewNewsletterSubscriber {
    pub email: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Search a newsletter subscriber by ID")]
pub struct SearchNewsletterSubscriberInput {
    pub subscriber_id: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Update a newsletter subscriber")]
pub struct NewsletterSubscriberMutation {
    pub subscriber_id: String,
    pub email: String,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a newsletter subscriber")]
pub struct DeleteNewsletterSubscriberInput {
    pub subscriber_id: String,
}
