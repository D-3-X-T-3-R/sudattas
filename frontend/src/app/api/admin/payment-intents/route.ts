import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const PAYMENT_INTENTS_ROOTS = ["searchPaymentIntent", "getPaymentIntent"];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: PAYMENT_INTENTS_ROOTS });
}
