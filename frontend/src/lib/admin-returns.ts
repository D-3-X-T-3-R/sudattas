import { gqlAdmin } from "./graphql-client";

export interface ReturnRequestItemRow {
  orderDetailId: string;
  quantity: string;
  refundAmountMinor: string;
  status: string;
}

export interface ReturnRequestRow {
  returnId: string;
  orderId: string;
  userId: string;
  /** "requested" | "approved" | "in_transit" | "received" | "refund_pending" | "refunded" | "rejected" | "cancelled" */
  status: string;
  reason: string;
  createdAt: string;
  receivedAt: string | null;
  refundAttemptId: string | null;
  items: ReturnRequestItemRow[];
}

const RETURN_FIELDS = `
  returnId
  orderId
  userId
  status
  reason
  createdAt
  receivedAt
  refundAttemptId
  items {
    orderDetailId
    quantity
    refundAmountMinor
    status
  }
`;

export async function fetchReturnRequestsAdmin(): Promise<ReturnRequestRow[]> {
  const data = await gqlAdmin<{ searchReturnRequests?: ReturnRequestRow[] }>(
    `query AdminSearchReturnRequests($input: SearchReturnRequestsInput!) {
      searchReturnRequests(input: $input) { ${RETURN_FIELDS} }
    }`,
    { input: {} }
  );
  const rows = data?.searchReturnRequests ?? [];
  return [...rows].sort((a, b) => Number(b.returnId) - Number(a.returnId));
}

export async function adminMarkReturnReceived(returnId: string): Promise<ReturnRequestRow | null> {
  const data = await gqlAdmin<{ adminMarkReturnReceived?: ReturnRequestRow[] }>(
    `mutation AdminMarkReturnReceived($input: AdminMarkReturnReceivedInput!) {
      adminMarkReturnReceived(input: $input) { ${RETURN_FIELDS} }
    }`,
    { input: { returnId } }
  );
  return data?.adminMarkReturnReceived?.[0] ?? null;
}

export async function adminUpdateReturnStatus(params: {
  returnId: string;
  status: "approved" | "in_transit" | "rejected" | "cancelled";
  note?: string;
}): Promise<ReturnRequestRow | null> {
  const data = await gqlAdmin<{ adminUpdateReturnStatus?: ReturnRequestRow[] }>(
    `mutation AdminUpdateReturnStatus($input: AdminUpdateReturnStatusInput!) {
      adminUpdateReturnStatus(input: $input) { ${RETURN_FIELDS} }
    }`,
    { input: params }
  );
  return data?.adminUpdateReturnStatus?.[0] ?? null;
}
