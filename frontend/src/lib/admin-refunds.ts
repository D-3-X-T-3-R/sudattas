import { gqlAdmin } from "./graphql-client";

export interface RefundRow {
  refundId: string;
  orderId: string;
  gatewayRefundId: string;
  amountPaise: string;
  currency: string;
  status: string;
  createdAt: string;
  lineItemsRefundedJson: string | null;
}

export interface RefundAttemptRow {
  attemptId: string;
  orderId: string;
  paymentIntentId: string | null;
  razorpayPaymentId: string | null;
  amountRequestedPaise: string;
  amountSentToGatewayPaise: string;
  gatewayRefundId: string | null;
  /** e.g. "pending_external" | "needs_review" | "resolved" */
  status: string;
  providerError: string | null;
  createdAt: string;
  updatedAt: string;
  attemptCount: number;
}

const REFUND_FIELDS = `
  refundId
  orderId
  gatewayRefundId
  amountPaise
  currency
  status
  createdAt
  lineItemsRefundedJson
`;

const REFUND_ATTEMPT_FIELDS = `
  attemptId
  orderId
  paymentIntentId
  razorpayPaymentId
  amountRequestedPaise
  amountSentToGatewayPaise
  gatewayRefundId
  status
  providerError
  createdAt
  updatedAt
  attemptCount
`;

export async function fetchRefundsAdmin(): Promise<RefundRow[]> {
  const data = await gqlAdmin<{ getRefunds?: RefundRow[] }>(
    `query AdminGetRefunds($input: GetRefund!) {
      getRefunds(input: $input) { ${REFUND_FIELDS} }
    }`,
    { input: {} }
  );
  const rows = data?.getRefunds ?? [];
  return [...rows].sort((a, b) => Number(b.refundId) - Number(a.refundId));
}

export async function createRefundAdmin(params: {
  orderId: string;
  gatewayRefundId: string;
  amountPaise: string;
  currency?: string;
}): Promise<RefundRow | null> {
  const data = await gqlAdmin<{ createRefund?: RefundRow[] }>(
    `mutation AdminCreateRefund($input: NewRefund!) {
      createRefund(input: $input) { ${REFUND_FIELDS} }
    }`,
    { input: params }
  );
  return data?.createRefund?.[0] ?? null;
}

export async function fetchRefundAttemptsAdmin(status?: string): Promise<RefundAttemptRow[]> {
  const data = await gqlAdmin<{ searchRefundAttempts?: RefundAttemptRow[] }>(
    `query AdminSearchRefundAttempts($input: SearchRefundAttemptsInput!) {
      searchRefundAttempts(input: $input) { ${REFUND_ATTEMPT_FIELDS} }
    }`,
    { input: status ? { status } : {} }
  );
  const rows = data?.searchRefundAttempts ?? [];
  return [...rows].sort((a, b) => Number(b.attemptId) - Number(a.attemptId));
}

export async function resolveRefundAttemptNeedsReview(params: {
  attemptId: string;
  resolution: "retry" | "mark_settled";
  actorId: string;
}): Promise<boolean> {
  const data = await gqlAdmin<{ resolveRefundAttemptNeedsReview?: boolean }>(
    `mutation AdminResolveRefundAttemptNeedsReview($input: ResolveRefundAttemptNeedsReviewInput!) {
      resolveRefundAttemptNeedsReview(input: $input)
    }`,
    { input: params }
  );
  return data?.resolveRefundAttemptNeedsReview === true;
}
