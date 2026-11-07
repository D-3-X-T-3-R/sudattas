import { gqlAdmin } from "./graphql-client";

export interface PaymentIntentRow {
  intentId: string;
  razorpayOrderId: string;
  orderId: string | null;
  userId: string | null;
  amountPaise: string;
  currency: string | null;
  /** "pending" | "processed" | "failed" | "needs_review" | "client_verified" */
  status: string;
  razorpayPaymentId: string | null;
  createdAt: string;
  expiresAt: string;
  gatewayFeePaise: string | null;
  gatewayTaxPaise: string | null;
}

const PAYMENT_INTENT_FIELDS = `
  intentId
  razorpayOrderId
  orderId
  userId
  amountPaise
  currency
  status
  razorpayPaymentId
  createdAt
  expiresAt
  gatewayFeePaise
  gatewayTaxPaise
`;

/** Admin search/browse — unlike a single getPaymentIntent lookup, this doesn't require already
 * knowing the intent/order id. Leave all params unset to list the most recent intents. */
export async function searchPaymentIntentsAdmin(params: {
  orderId?: string;
  userId?: string;
  razorpayOrderId?: string;
  razorpayPaymentId?: string;
  status?: string;
  limit?: string;
} = {}): Promise<PaymentIntentRow[]> {
  const input: Record<string, string> = {};
  if (params.orderId) input.orderId = params.orderId;
  if (params.userId) input.userId = params.userId;
  if (params.razorpayOrderId) input.razorpayOrderId = params.razorpayOrderId;
  if (params.razorpayPaymentId) input.razorpayPaymentId = params.razorpayPaymentId;
  if (params.status) input.status = params.status;
  input.limit = params.limit ?? "50";

  const data = await gqlAdmin<{ searchPaymentIntent?: PaymentIntentRow[] }>(
    `query AdminSearchPaymentIntent($input: SearchPaymentIntent!) {
      searchPaymentIntent(input: $input) { ${PAYMENT_INTENT_FIELDS} }
    }`,
    { input }
  );
  return data?.searchPaymentIntent ?? [];
}
