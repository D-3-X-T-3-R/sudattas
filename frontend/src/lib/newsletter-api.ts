/**
 * Storefront footer newsletter signup, and the public one-click unsubscribe link embedded in
 * every campaign email. Both public — no login or guest cart required beyond the guest
 * session `gql()` already establishes for every anonymous visitor.
 */
import { gql, GraphqlRequestError } from "@/lib/graphqlClient";

const SUBSCRIBE_NEWSLETTER_MUTATION = `mutation SubscribeNewsletter($email: String!) {
  subscribeNewsletter(email: $email)
}`;

const UNSUBSCRIBE_NEWSLETTER_MUTATION = `mutation UnsubscribeNewsletter($input: UnsubscribeNewsletterInput!) {
  unsubscribeNewsletter(input: $input)
}`;

/** Thrown with a message safe to show directly to the visitor. */
export class NewsletterSignupError extends Error {}

/**
 * Subscribe an email to the newsletter. Resolves on success; rejects with
 * `NewsletterSignupError` carrying a user-facing message (e.g. already subscribed,
 * invalid address) on failure.
 */
export async function subscribeToNewsletter(email: string): Promise<void> {
  try {
    await gql<{ subscribeNewsletter: boolean }>(SUBSCRIBE_NEWSLETTER_MUTATION, { email });
  } catch (err) {
    if (err instanceof GraphqlRequestError && err.code === "AlreadyExists") {
      throw new NewsletterSignupError("You're already on the list with that email.");
    }
    const message = err instanceof Error ? err.message : "Could not subscribe. Please try again.";
    throw new NewsletterSignupError(message);
  }
}

/** Thrown with a message safe to show directly on the unsubscribe landing page. */
export class NewsletterUnsubscribeError extends Error {}

/**
 * Unsubscribe via the signed link from a campaign email. Idempotent on the backend — clicking
 * an already-processed link resolves normally rather than erroring.
 */
export async function unsubscribeFromNewsletter(subscriberId: string, token: string): Promise<void> {
  try {
    await gql<{ unsubscribeNewsletter: boolean }>(UNSUBSCRIBE_NEWSLETTER_MUTATION, {
      input: { subscriberId, token },
    });
  } catch (err) {
    const message =
      err instanceof Error ? err.message : "This unsubscribe link is invalid or has expired.";
    throw new NewsletterUnsubscribeError(message);
  }
}
