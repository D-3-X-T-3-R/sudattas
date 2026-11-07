import { gqlAdmin } from "./graphql-client";

/** One line item on an admin order detail view. */
export interface AdminOrderDetailLine {
  orderDetailId: string;
  variantId: string;
  quantity: string;
  priceFormatted: string;
  productName: string | null;
  productId: string | null;
}

/** One entry in an order's audit timeline. */
export interface AdminOrderEvent {
  eventId: string;
  eventType: string;
  fromStatus: string;
  toStatus: string;
  /** customer | admin | system */
  actorType: string;
  message: string;
  createdAt: string;
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
  events: AdminOrderEvent[];
  invoiceAvailable: boolean;
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
    invoiceAvailable
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
    eventId
    eventType
    fromStatus
    toStatus
    actorType
    message
    createdAt
  }
}`;

function deriveRefundTrackingState(
  events: Array<{ eventType: string }> | undefined
): "none" | "initiated" | "processed" | "failed" {
  const eventTypes = (events ?? []).map((x) => (x.eventType ?? "").trim().toLowerCase());
  if (eventTypes.some((x) => x === "refund_failed")) return "failed";
  // "refund_recorded" covers a completed real-gateway refund; "refund_recorded_admin_settled"
  // is the same completed state for an order with no online payment on file (e.g. a manually
  // created order) — cancelling it is treated as the admin having already refunded the
  // customer directly, so both land in the same "processed" bucket.
  if (eventTypes.some((x) => x === "refund_recorded" || x === "refund_recorded_admin_settled")) {
    return "processed";
  }
  // "refund_initiated" fires once the gateway confirms the refund; "refund_pending_external"
  // fires earlier, the moment the attempt is queued for the background worker — both mean a
  // real-gateway refund is in motion.
  if (eventTypes.some((x) => x === "refund_initiated" || x === "refund_pending_external")) {
    return "initiated";
  }
  return "none";
}

/** Load one order by id with line items (admin). Returns null if not found. */
export async function fetchAdminOrderById(orderId: string): Promise<AdminOrderDetail | null> {
  const id = orderId.trim();
  if (!id) return null;
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
        invoiceAvailable: boolean;
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
    gqlAdmin<{ getOrderEvents?: AdminOrderEvent[] }>(ADMIN_ORDER_EVENTS_QUERY, {
      orderId: id,
    }),
  ]);
  const row = data?.searchOrder?.[0];
  if (!row) {
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
  const events = [...(eventsData?.getOrderEvents ?? [])].sort(
    (a, b) => new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
  );
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
    events,
    invoiceAvailable: row.invoiceAvailable,
  };
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

const RESOLVE_NEEDS_REVIEW_MUTATION = `mutation AdminResolveNeedsReview($input: ResolveNeedsReviewInput!) {
  resolveNeedsReview(input: $input)
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
  if (options?.shiprocketBook) {
    await gqlAdmin(ADMIN_MARK_ORDER_SHIPPED_MUTATION, {
      input: {
        orderId: order.orderId,
        shiprocketBook: true,
      },
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
}

const DELETE_ORDER_MUTATION = `mutation AdminCancelOrder($orderId: String!) {
  deleteOrder(orderId: $orderId) {
    orderId
    statusId
  }
}`;

/**
 * Admin: cancel an order (status → "cancelled"), bypassing the ownership check customers get
 * (admins may cancel any order). Unlike picking "Cancelled" from the raw status dropdown
 * (updateOrder), this goes through the same path a customer's own cancellation uses: it checks
 * the cancel window and fulfillment status, and restores inventory for the cancelled lines.
 * Rejects with a clear reason once the window has closed or a shipment is already in motion —
 * at that point, use returns/refuse-delivery instead.
 */
export async function cancelOrderAdmin(orderId: string): Promise<void> {
  await gqlAdmin(DELETE_ORDER_MUTATION, { orderId });
}

/**
 * Admin: resolve an order stuck in `needs_review` (e.g. an ambiguous payment-webhook outcome) by
 * marking it paid, cancelled, or refunded. Unlike the generic status dropdown (updateOrder), this
 * goes through the backend's dedicated resolution path, which also sets payment_status correctly
 * per outcome (core_operations::handlers::orders::resolve_needs_review) — a plain updateOrder call
 * would change status_id without touching payment_status, leaving it stale.
 */
export async function resolveOrderNeedsReview(
  orderId: string,
  resolution: "paid" | "cancelled" | "refunded"
): Promise<void> {
  await gqlAdmin(RESOLVE_NEEDS_REVIEW_MUTATION, {
    input: {
      orderId,
      resolution,
      actorId: "admin",
    },
  });
}

const CREATE_ORDER_EVENT_MUTATION = `mutation AdminCreateOrderEvent($input: NewOrderEvent!) {
  createOrderEvent(input: $input) {
    eventId
    eventType
    fromStatus
    toStatus
    actorType
    message
    createdAt
  }
}`;

/** Admin: add a manual note to an order's timeline (event_type "admin_note", actor_type "admin"). */
export async function createAdminOrderNote(
  orderId: string,
  message: string
): Promise<AdminOrderEvent> {
  const data = await gqlAdmin<{ createOrderEvent?: AdminOrderEvent[] }>(
    CREATE_ORDER_EVENT_MUTATION,
    {
      input: {
        orderId,
        eventType: "admin_note",
        actorType: "admin",
        message: message.trim(),
      },
    }
  );
  const created = data?.createOrderEvent?.[0];
  if (!created) {
    throw new Error("createOrderEvent returned empty payload");
  }
  return created;
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
