import { gqlAdmin } from "./graphql-client";

/**
 * Admin: manually create an order record. This is a pure record-creation tool — unlike the
 * customer-facing checkout (place_order), create_order_admin does NOT decrement inventory, does
 * NOT charge payment, and does NOT send any confirmation email/notification. It just persists an
 * Orders row (plus, via a second call, OrderDetails rows) with whatever totals the caller
 * computed. Useful for phone orders, gifts, or backfilling — not a checkout replacement.
 */

export interface AdminShippingAddressRow {
  shippingAddressId: string;
  userId: string | null;
  isDefault: boolean;
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road: string | null;
  apartmentNoOrName: string | null;
  recipientName: string | null;
  phoneNumber: string | null;
}

const SHIPPING_ADDRESS_FIELDS = `
  shippingAddressId
  userId
  isDefault
  country
  stateRegion
  city
  postalCode
  road
  apartmentNoOrName
  recipientName
  phoneNumber
`;

/** All shipping addresses across all customers — get_shipping_addresses returns everything,
 * unfiltered, when the caller is an admin (only non-admin callers get self-scoped results). */
export async function fetchAllShippingAddressesAdmin(): Promise<AdminShippingAddressRow[]> {
  const data = await gqlAdmin<{ getShippingAddresses?: AdminShippingAddressRow[] }>(
    `query AdminAllShippingAddresses { getShippingAddresses { ${SHIPPING_ADDRESS_FIELDS} } }`
  );
  return data?.getShippingAddresses ?? [];
}

export interface NewShippingAddressInput {
  userId: string;
  country: string;
  stateRegion: string;
  city: string;
  postalCode: string;
  road?: string;
  apartmentNoOrName?: string;
  recipientName?: string;
  phoneNumber?: string;
  isDefault?: boolean;
}

/** Create a shipping address on behalf of a customer (admin can set userId explicitly — only
 * non-admin callers are forced to their own JWT user id). */
export async function createShippingAddressAdmin(
  input: NewShippingAddressInput
): Promise<AdminShippingAddressRow | null> {
  const data = await gqlAdmin<{ createShippingAddress?: AdminShippingAddressRow[] }>(
    `mutation AdminCreateShippingAddress($input: NewShippingAddress!) {
      createShippingAddress(input: $input) { ${SHIPPING_ADDRESS_FIELDS} }
    }`,
    { input }
  );
  return data?.createShippingAddress?.[0] ?? null;
}

export interface CreateOrderAdminInput {
  userId: string;
  shippingAddressId: string;
  statusId: string;
  totalAmountPaise: string;
  subtotalMinor: string;
  shippingMinor: string;
  grandTotalMinor: string;
}

/** Create the bare order shell. Returns the new order's id. */
export async function createOrderAdmin(input: CreateOrderAdminInput): Promise<string> {
  const data = await gqlAdmin<{ createOrderAdmin?: Array<{ orderId: string }> }>(
    `mutation AdminCreateOrder($input: CreateOrderInput!) {
      createOrderAdmin(input: $input) { orderId }
    }`,
    { input }
  );
  const orderId = data?.createOrderAdmin?.[0]?.orderId;
  if (!orderId) {
    throw new Error("createOrderAdmin returned no order id");
  }
  return orderId;
}

export interface NewOrderDetailLine {
  variantId: string;
  quantity: string;
  /** Line total in paise (unit price × quantity), not unit price. */
  pricePaise: string;
}

/** Attach line items to an order just created via createOrderAdmin. */
export async function createOrderDetailsAdmin(
  orderId: string,
  lines: NewOrderDetailLine[]
): Promise<void> {
  await gqlAdmin<{ createOrderDetails?: unknown[] }>(
    `mutation AdminCreateOrderDetails($orderDetails: NewOrderDetails!) {
      createOrderDetails(orderDetails: $orderDetails) { orderDetailId }
    }`,
    {
      orderDetails: {
        orderDetails: lines.map((l) => ({ orderId, ...l })),
      },
    }
  );
}
