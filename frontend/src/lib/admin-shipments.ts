import { gqlAdmin } from "./graphql-client";

export interface AdminShipmentRow {
  shipmentId: string;
  orderId: string;
  shiprocketOrderId: string | null;
  awbCode: string | null;
  carrier: string | null;
  /** pending | processed | failed */
  status: string;
  createdAt: string;
  deliveredAt: string | null;
  /** JSON array of courier milestones, e.g. [{"label":"Picked up","at":"...","location":"..."}]. */
  trackingEventsJson: string | null;
  shiprocketStatusId: string | null;
  shiprocketStatusLabel: string | null;
  customerTrackingStatus: string;
}

const SHIPMENT_FIELDS = `
  shipmentId
  orderId
  shiprocketOrderId
  awbCode
  carrier
  status
  createdAt
  deliveredAt
  trackingEventsJson
  shiprocketStatusId
  shiprocketStatusLabel
  customerTrackingStatus
`;

export async function fetchShipmentsForOrder(orderId: string): Promise<AdminShipmentRow[]> {
  const data = await gqlAdmin<{ getShipment?: AdminShipmentRow[] }>(
    `query AdminGetShipment($input: GetShipment!) {
      getShipment(input: $input) { ${SHIPMENT_FIELDS} }
    }`,
    { input: { orderId } }
  );
  return data?.getShipment ?? [];
}

export async function createShipmentAdmin(params: {
  orderId: string;
  shiprocketOrderId?: string;
  awbCode?: string;
  carrier?: string;
}): Promise<AdminShipmentRow | null> {
  const data = await gqlAdmin<{ createShipment?: AdminShipmentRow[] }>(
    `mutation AdminCreateShipment($input: NewShipment!) {
      createShipment(input: $input) { ${SHIPMENT_FIELDS} }
    }`,
    { input: params }
  );
  return data?.createShipment?.[0] ?? null;
}

export async function updateShipmentAdmin(params: {
  shipmentId: string;
  awbCode?: string;
  carrier?: string;
  status?: string;
}): Promise<AdminShipmentRow | null> {
  const data = await gqlAdmin<{ updateShipment?: AdminShipmentRow[] }>(
    `mutation AdminUpdateShipment($input: UpdateShipment!) {
      updateShipment(input: $input) { ${SHIPMENT_FIELDS} }
    }`,
    { input: params }
  );
  return data?.updateShipment?.[0] ?? null;
}

/** Pull the latest status/tracking events from Shiprocket for every shipment on this order. */
export async function syncShipmentsFromShiprocketAdmin(orderId: string): Promise<AdminShipmentRow[]> {
  const data = await gqlAdmin<{ syncOrderShipmentsFromShiprocket?: AdminShipmentRow[] }>(
    `mutation AdminSyncShipments($orderId: String!) {
      syncOrderShipmentsFromShiprocket(orderId: $orderId) { ${SHIPMENT_FIELDS} }
    }`,
    { orderId }
  );
  return data?.syncOrderShipmentsFromShiprocket ?? [];
}

/** Best-effort parse of trackingEventsJson into a display list; falls back to empty on any
 * malformed/legacy shape rather than throwing. */
export interface TrackingEvent {
  label?: string;
  at?: string;
  location?: string;
}

export function parseTrackingEvents(json: string | null): TrackingEvent[] {
  if (!json) return [];
  try {
    const parsed: unknown = JSON.parse(json);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (e): e is TrackingEvent => typeof e === "object" && e !== null
    );
  } catch {
    return [];
  }
}
