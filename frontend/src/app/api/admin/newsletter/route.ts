import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const NEWSLETTER_ROOTS = [
  "searchNewsletterSubscriber",
  "createNewsletterSubscriber",
  "updateNewsletterSubscriber",
  "deleteNewsletterSubscriber",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: NEWSLETTER_ROOTS });
}
