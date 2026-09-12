import { gqlAdmin } from "./graphql-client";

/**
 * Admin: place an order on a customer's behalf. `placeOrderAdmin` (below) mirrors real checkout's
 * COD path exactly — order creation, immediate order_state_machine transition to "confirmed",
 * payment capture, invoice generation, and an order_event — which also means it enqueues the same
 * PAYMENT_CAPTURED outbox event real orders do, so the customer *does* get the usual confirmation
 * notification. The only things it still doesn't do: decrement inventory, or run a live Razorpay
 * step (both payment methods are always treated as already settled). The older
 * createOrderAdmin + createOrderDetailsAdmin two-call flow below is lower-level — it leaves the
 * order at whatever raw status the caller picks, with none of the above side effects.
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
  /** "cod" | "prepaid". "prepaid" asserts payment was already collected outside this system
   * (cash/UPI/bank transfer) — it's recorded as captured immediately, no live payment flow.
   * Required: without it, the order can never pass the shipment-booking eligibility check. */
  paymentMethod: "cod" | "prepaid";
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

export interface PlaceOrderAdminLineItem {
  variantId: string;
  quantity: string;
  /** Line total in paise (unit price × quantity), not unit price. */
  pricePaise: string;
}

export interface PlaceOrderAdminInput {
  userId: string;
  shippingAddressId: string;
  /** "cod" | "prepaid". Both are treated as already settled — there is never a live Razorpay
   * step for an admin-placed order. "prepaid" only records that payment happened outside this
   * system, it does not collect it. */
  paymentMethod: "cod" | "prepaid";
  lineItems: PlaceOrderAdminLineItem[];
  shippingMinor?: string;
}

/**
 * Admin: place a full order on a customer's behalf in one atomic call — order, line items, and
 * immediate confirm/capture/invoice/order_event, mirroring exactly what real COD checkout does
 * at placement (same order_state_machine transition to "confirmed", same cancel window and
 * shipment-booking eligibility). Replaces the old createOrderAdmin + createOrderDetailsAdmin
 * two-step flow, which left the order stuck at an admin-picked raw status with no payment
 * capture, no invoice, and no order_event — never actually reaching parity with a real order.
 */
export async function placeOrderAdmin(input: PlaceOrderAdminInput): Promise<string> {
  const data = await gqlAdmin<{ placeOrderAdmin?: Array<{ orderId: string }> }>(
    `mutation AdminPlaceOrder($input: PlaceOrderAdminInput!) {
      placeOrderAdmin(input: $input) { orderId }
    }`,
    { input }
  );
  const orderId = data?.placeOrderAdmin?.[0]?.orderId;
  if (!orderId) {
    throw new Error("placeOrderAdmin returned no order id");
  }
  return orderId;
}
