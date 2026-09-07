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

/** One past send of a newsletter campaign — immutable once recorded, no draft/edit. */
export interface NewsletterCampaignRow {
  campaignId: string;
  subject: string;
  bodyText: string;
  ctaLabel: string;
  ctaUrl: string;
  recipientCount: string;
  successCount: string;
  failureCount: string;
  sentAt: string;
}

const CAMPAIGN_FIELDS = `campaignId subject bodyText ctaLabel ctaUrl recipientCount successCount failureCount sentAt`;

export async function fetchNewsletterCampaigns(): Promise<NewsletterCampaignRow[]> {
  const data = await gqlAdmin<{ searchNewsletterCampaign?: NewsletterCampaignRow[] }>(
    `mutation NewsletterCampaigns {
      searchNewsletterCampaign(input: {}) { ${CAMPAIGN_FIELDS} }
    }`
  );
  return data?.searchNewsletterCampaign ?? [];
}

export interface SendNewsletterCampaignInput {
  subject: string;
  bodyText: string;
  ctaLabel?: string;
  ctaUrl?: string;
}

/**
 * Compose and immediately send a campaign to every subscriber who hasn't unsubscribed.
 * There is no draft/preview step server-side — this sends for real. Resolves with the
 * recorded campaign row (including final success/failure counts) once every recipient has
 * been attempted, which for a large list can take a while (Resend calls are throttled
 * server-side to stay under its rate limit).
 */
export async function sendNewsletterCampaign(
  input: SendNewsletterCampaignInput
): Promise<NewsletterCampaignRow> {
  const data = await gqlAdmin<{ sendNewsletterCampaign?: NewsletterCampaignRow[] }>(
    `mutation SendNewsletterCampaign($input: SendNewsletterCampaignInput!) {
      sendNewsletterCampaign(input: $input) { ${CAMPAIGN_FIELDS} }
    }`,
    {
      input: {
        subject: input.subject,
        bodyText: input.bodyText,
        ctaLabel: input.ctaLabel?.trim() || undefined,
        ctaUrl: input.ctaUrl?.trim() || undefined,
      },
    }
  );
  const row = data?.sendNewsletterCampaign?.[0];
  if (!row) {
    throw new Error("sendNewsletterCampaign returned empty payload");
  }
  return row;
}
