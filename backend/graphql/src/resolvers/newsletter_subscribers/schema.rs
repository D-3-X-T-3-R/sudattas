use juniper::{graphql_object, GraphQLInputObject};

#[derive(Default, Debug, Clone)]
pub struct NewsletterSubscriber {
    pub subscriber_id: String,
    pub email: String,
    pub subscription_date: String,
    /// RFC3339; empty if still subscribed.
    pub unsubscribed_at: String,
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

    async fn unsubscribed_at(&self) -> &String {
        &self.unsubscribed_at
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
    /// Set true to unsubscribe, false to resubscribe; omit to leave unchanged.
    pub unsubscribed: Option<bool>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Delete a newsletter subscriber")]
pub struct DeleteNewsletterSubscriberInput {
    pub subscriber_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct NewsletterCampaign {
    pub campaign_id: String,
    pub subject: String,
    pub body_text: String,
    pub cta_label: String,
    pub cta_url: String,
    pub recipient_count: String,
    pub success_count: String,
    pub failure_count: String,
    pub sent_at: String,
}

#[graphql_object]
#[graphql(description = "One send of a newsletter campaign to all active subscribers")]
impl NewsletterCampaign {
    async fn campaign_id(&self) -> &String {
        &self.campaign_id
    }
    async fn subject(&self) -> &String {
        &self.subject
    }
    async fn body_text(&self) -> &String {
        &self.body_text
    }
    async fn cta_label(&self) -> &String {
        &self.cta_label
    }
    async fn cta_url(&self) -> &String {
        &self.cta_url
    }
    async fn recipient_count(&self) -> &String {
        &self.recipient_count
    }
    async fn success_count(&self) -> &String {
        &self.success_count
    }
    async fn failure_count(&self) -> &String {
        &self.failure_count
    }
    async fn sent_at(&self) -> &String {
        &self.sent_at
    }
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(
    description = "Compose and immediately send a newsletter campaign to every subscriber who hasn't unsubscribed"
)]
pub struct SendNewsletterCampaignInput {
    pub subject: String,
    pub body_text: String,
    pub cta_label: Option<String>,
    pub cta_url: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Look up past newsletter campaigns; omit campaignId to list all")]
pub struct SearchNewsletterCampaignInput {
    pub campaign_id: Option<String>,
}

#[derive(GraphQLInputObject, Default, Debug)]
#[graphql(description = "Unsubscribe via the signed link in a campaign email")]
pub struct UnsubscribeNewsletterInput {
    pub subscriber_id: String,
    pub token: String,
}
