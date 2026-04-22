import { gqlAdmin } from "./graphql-client";

function flowLog(message: string, meta?: Record<string, unknown>) {
  if (meta) {
    console.info(`[orders-flow][admin-ui] ${message}`, meta);
    return;
  }
  console.info(`[orders-flow][admin-ui] ${message}`);
}

/** One line item on an admin order detail view. */
export interface AdminOrderDetailLine {
  orderDetailId: string;
  variantId: string;
  quantity: string;
  priceFormatted: string;
  productName: string | null;
  productId: string | null;
}

/** Full order + lines for admin detail page. */
export interface AdminOrderDetail {
  orderId: string;
  userId: string;
  orderDate: string;
  cancelWindowEndsAt?: string | null;
  earliestBookingAt?: string | null;
  pickupTargetAt?: string | null;
  fulfillmentStatus?: string | null;
  shippingAddressId: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  lines: AdminOrderDetailLine[];
  refundTrackingState: "none" | "initiated" | "processed" | "failed";
}

const ADMIN_ORDER_DETAIL_QUERY = `query AdminOrderDetail($search: SearchOrder!) {
  searchOrder(search: $search) {
    orderId
    userId
    orderDate
    cancelWindowEndsAt
    earliestBookingAt
    pickupTargetAt
    fulfillmentStatus
    shippingAddressId
    totalAmountPaise
    totalAmountFormatted
    statusId
    orderDetails {
      orderDetailId
      variantId
      quantity
      priceFormatted
      productDetails {
        productId
        name
      }
    }
  }
}`;

const ADMIN_ORDER_EVENTS_QUERY = `query AdminOrderEvents($orderId: String!) {
  getOrderEvents(orderId: $orderId) {
    eventType
  }
}`;

function deriveRefundTrackingState(
  events: Array<{ eventType: string }> | undefined
): "none" | "initiated" | "processed" | "failed" {
  const eventTypes = (events ?? []).map((x) => (x.eventType ?? "").trim().toLowerCase());
  if (eventTypes.some((x) => x === "refund_failed")) return "failed";
  if (eventTypes.some((x) => x === "refund_recorded")) return "processed";
  if (eventTypes.some((x) => x === "refund_initiated")) return "initiated";
  return "none";
}

/** Load one order by id with line items (admin). Returns null if not found. */
export async function fetchAdminOrderById(orderId: string): Promise<AdminOrderDetail | null> {
  const id = orderId.trim();
  if (!id) return null;
  flowLog("fetching admin order detail", { orderId: id });
  const [data, eventsData] = await Promise.all([
    gqlAdmin<{
      searchOrder?: Array<{
        orderId: string;
        userId: string;
        orderDate: string;
        cancelWindowEndsAt?: string | null;
        earliestBookingAt?: string | null;
        pickupTargetAt?: string | null;
        fulfillmentStatus?: string | null;
        shippingAddressId: string;
        totalAmountPaise: string;
        totalAmountFormatted: string;
        statusId: string;
        orderDetails: Array<{
          orderDetailId: string;
          variantId: string;
          quantity: string;
          priceFormatted: string;
          productDetails: Array<{ productId: string; name: string }>;
        }>;
      }>;
    }>(ADMIN_ORDER_DETAIL_QUERY, {
      search: { userId: "", orderId: id, limit: "1" },
    }),
    gqlAdmin<{ getOrderEvents?: Array<{ eventType: string }> }>(ADMIN_ORDER_EVENTS_QUERY, {
      orderId: id,
    }),
  ]);
  const row = data?.searchOrder?.[0];
  if (!row) {
    flowLog("admin order detail not found", { orderId: id });
    return null;
  }
  const lines: AdminOrderDetailLine[] = (row.orderDetails ?? []).map((d) => {
    const p = d.productDetails?.[0];
    return {
      orderDetailId: d.orderDetailId,
      variantId: d.variantId,
      quantity: d.quantity,
      priceFormatted: d.priceFormatted,
      productName: p?.name ?? null,
      productId: p?.productId ?? null,
    };
  });
  const refundTrackingState = deriveRefundTrackingState(eventsData?.getOrderEvents);
  const result = {
    orderId: row.orderId,
    userId: row.userId,
    orderDate: row.orderDate,
    cancelWindowEndsAt: row.cancelWindowEndsAt ?? null,
    earliestBookingAt: row.earliestBookingAt ?? null,
    pickupTargetAt: row.pickupTargetAt ?? null,
    fulfillmentStatus: row.fulfillmentStatus ?? null,
    shippingAddressId: row.shippingAddressId,
    totalAmountPaise: String(row.totalAmountPaise ?? ""),
    totalAmountFormatted: row.totalAmountFormatted,
    statusId: row.statusId,
    lines,
    refundTrackingState,
  };
  flowLog("admin order detail loaded", {
    orderId: result.orderId,
    userId: result.userId,
    statusId: result.statusId,
    lineCount: result.lines.length,
    refundTrackingState,
  });
  return result;
}

const UPDATE_ORDER_MUTATION = `mutation UpdateAdminOrder($order: OrderMutation!) {
  updateOrder(order: $order) {
    orderId
    statusId
  }
}`;

const ADMIN_MARK_ORDER_SHIPPED_MUTATION = `mutation AdminMarkOrderShipped($input: AdminMarkOrderShippedInput!) {
  adminMarkOrderShipped(input: $input)
}`;

const UPDATE_PICKUP_TARGET_MUTATION = `mutation UpdatePickupTarget($input: UpdatePickupTargetInput!) {
  updatePickupTarget(input: $input) {
    orderId
    pickupTargetAt
    pickupTargetReason
    pickupTargetSetBy
    pickupTargetUpdatedAt
  }
}`;

/**
 * Admin: change order status.
 * For "shipped", callers can opt into Shiprocket booking by setting `shiprocketBook`.
 * Other statuses use updateOrder.
 */
export async function updateAdminOrderStatus(
  order: AdminOrderDetail,
  newStatusId: string,
  options?: { shiprocketBook?: boolean }
): Promise<void> {
  flowLog("admin status update requested", {
    orderId: order.orderId,
    currentStatusId: order.statusId,
    targetStatusId: newStatusId.trim(),
    shiprocketBook: Boolean(options?.shiprocketBook),
  });
  if (options?.shiprocketBook) {
    await gqlAdmin(ADMIN_MARK_ORDER_SHIPPED_MUTATION, {
      input: {
        orderId: order.orderId,
        shiprocketBook: true,
      },
    });
    flowLog("admin shipped mutation completed", {
      orderId: order.orderId,
      targetStatusId: newStatusId.trim(),
    });
    return;
  }

  await gqlAdmin(UPDATE_ORDER_MUTATION, {
    order: {
      orderId: order.orderId,
      userId: order.userId,
      orderDate: order.orderDate || new Date().toISOString(),
      shippingAddressId: order.shippingAddressId,
      totalAmountPaise: String(order.totalAmountPaise ?? ""),
      statusId: newStatusId.trim(),
    },
  });
  flowLog("admin updateOrder mutation completed", {
    orderId: order.orderId,
    targetStatusId: newStatusId.trim(),
  });
}

export async function updateAdminPickupTarget(params: {
  orderId: string;
  pickupTargetAt: string;
  reason?: string;
}): Promise<{
  orderId: string;
  pickupTargetAt: string;
  pickupTargetReason?: string | null;
  pickupTargetSetBy?: string | null;
  pickupTargetUpdatedAt: string;
}> {
  const data = await gqlAdmin<{
    updatePickupTarget?: {
      orderId: string;
      pickupTargetAt: string;
      pickupTargetReason?: string | null;
      pickupTargetSetBy?: string | null;
      pickupTargetUpdatedAt: string;
    };
  }>(UPDATE_PICKUP_TARGET_MUTATION, {
    input: {
      orderId: params.orderId,
      pickupTargetAt: params.pickupTargetAt,
      reason: params.reason?.trim() || undefined,
    },
  });
  if (!data?.updatePickupTarget) {
    throw new Error("updatePickupTarget returned empty payload");
  }
  return data.updatePickupTarget;
}
