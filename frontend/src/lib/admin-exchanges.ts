import { gqlAdmin } from "./graphql-client";

export interface ExchangeRequestRow {
  exchangeId: string;
  orderId: string;
  userId: string;
  orderDetailId: string;
  desiredVariantId: string;
  quantity: string;
  /** "requested" | "approved" | "in_transit" | "received" | "completed" | "rejected" | "cancelled" */
  status: string;
  reason: string;
  createdAt: string;
  receivedAt: string | null;
  replacementOrderId: string | null;
  pickupShiprocketOrderId: string | null;
  pickupShiprocketShipmentId: string | null;
  pickupAwbCode: string | null;
  pickupCourierName: string | null;
  pickupStatus: string | null;
  pickupScheduledAt: string | null;
  pickupTrackingEventsJson: string | null;
}

const EXCHANGE_FIELDS = `
  exchangeId
  orderId
  userId
  orderDetailId
  desiredVariantId
  quantity
  status
  reason
  createdAt
  receivedAt
  replacementOrderId
  pickupShiprocketOrderId
  pickupShiprocketShipmentId
  pickupAwbCode
  pickupCourierName
  pickupStatus
  pickupScheduledAt
  pickupTrackingEventsJson
`;

export async function fetchExchangeRequestsAdmin(): Promise<ExchangeRequestRow[]> {
  const data = await gqlAdmin<{ searchExchangeRequests?: ExchangeRequestRow[] }>(
    `query AdminSearchExchangeRequests($input: SearchExchangeRequestsInput!) {
      searchExchangeRequests(input: $input) { ${EXCHANGE_FIELDS} }
    }`,
    { input: {} }
  );
  const rows = data?.searchExchangeRequests ?? [];
  return [...rows].sort((a, b) => Number(b.exchangeId) - Number(a.exchangeId));
}

export async function adminMarkExchangeReceived(
  exchangeId: string
): Promise<ExchangeRequestRow | null> {
  const data = await gqlAdmin<{ adminMarkExchangeReceived?: ExchangeRequestRow[] }>(
    `mutation AdminMarkExchangeReceived($input: AdminMarkExchangeReceivedInput!) {
      adminMarkExchangeReceived(input: $input) { ${EXCHANGE_FIELDS} }
    }`,
    { input: { exchangeId } }
  );
  return data?.adminMarkExchangeReceived?.[0] ?? null;
}

export async function adminUpdateExchangeStatus(params: {
  exchangeId: string;
  status: "approved" | "in_transit" | "rejected" | "cancelled";
  note?: string;
}): Promise<ExchangeRequestRow | null> {
  const data = await gqlAdmin<{ adminUpdateExchangeStatus?: ExchangeRequestRow[] }>(
    `mutation AdminUpdateExchangeStatus($input: AdminUpdateExchangeStatusInput!) {
      adminUpdateExchangeStatus(input: $input) { ${EXCHANGE_FIELDS} }
    }`,
    { input: params }
  );
  return data?.adminUpdateExchangeStatus?.[0] ?? null;
}

/** Books the reverse-pickup shipment via Shiprocket — a real courier request, only ever
 * triggered by this explicit action, never automatically on approve. */
export async function scheduleExchangePickup(
  exchangeId: string
): Promise<ExchangeRequestRow | null> {
  const data = await gqlAdmin<{ scheduleExchangePickup?: ExchangeRequestRow[] }>(
    `mutation ScheduleExchangePickup($input: ScheduleExchangePickupInput!) {
      scheduleExchangePickup(input: $input) { ${EXCHANGE_FIELDS} }
    }`,
    { input: { exchangeId } }
  );
  return data?.scheduleExchangePickup?.[0] ?? null;
}

/** Refreshes the reverse-pickup tracking; auto-completes the exchange (stock restored,
 * replacement order created) once Shiprocket reports it delivered to the warehouse. */
export async function syncExchangePickup(
  exchangeId: string
): Promise<ExchangeRequestRow | null> {
  const data = await gqlAdmin<{ syncExchangePickup?: ExchangeRequestRow[] }>(
    `mutation SyncExchangePickup($input: SyncExchangePickupInput!) {
      syncExchangePickup(input: $input) { ${EXCHANGE_FIELDS} }
    }`,
    { input: { exchangeId } }
  );
  return data?.syncExchangePickup?.[0] ?? null;
}
