import { gqlAdmin } from "./graphql-client";

export interface NewsletterSubscriberRow {
  subscriberId: string;
  email: string;
  subscriptionDate: string;
  /** RFC3339; empty if still subscribed. */
  unsubscribedAt: string;
}

const SUBSCRIBER_FIELDS = `subscriberId email subscriptionDate unsubscribedAt`;

export async function fetchNewsletterSubscribers(): Promise<NewsletterSubscriberRow[]> {
  const data = await gqlAdmin<{ searchNewsletterSubscriber?: NewsletterSubscriberRow[] }>(
    `mutation NewsletterSubscribers {
      searchNewsletterSubscriber(input: { subscriberId: "0" }) { ${SUBSCRIBER_FIELDS} }
    }`
  );
  const rows = data?.searchNewsletterSubscriber ?? [];
  return [...rows].sort(
    (a, b) => new Date(b.subscriptionDate).getTime() - new Date(a.subscriptionDate).getTime()
  );
}

export async function setNewsletterSubscriberUnsubscribed(
  subscriberId: string,
  email: string,
  unsubscribed: boolean
): Promise<NewsletterSubscriberRow | null> {
  const data = await gqlAdmin<{ updateNewsletterSubscriber?: NewsletterSubscriberRow[] }>(
    `mutation SetSubscriberUnsubscribed($input: NewsletterSubscriberMutation!) {
      updateNewsletterSubscriber(input: $input) { ${SUBSCRIBER_FIELDS} }
    }`,
    { input: { subscriberId, email, unsubscribed } }
  );
  return data?.updateNewsletterSubscriber?.[0] ?? null;
}

export async function deleteNewsletterSubscriberAdmin(subscriberId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteNewsletterSubscriber($input: DeleteNewsletterSubscriberInput!) {
      deleteNewsletterSubscriber(input: $input) { subscriberId }
    }`,
    { input: { subscriberId } }
  );
}
