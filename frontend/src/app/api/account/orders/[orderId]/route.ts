import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

function flowLog(message: string, meta?: Record<string, unknown>) {
  if (meta) {
    console.info(`[orders-flow][customer-api] ${message}`, meta);
    return;
  }
  console.info(`[orders-flow][customer-api] ${message}`);
}

type OrderDetailRow = {
  orderDetailId: string;
  variantId: string;
  quantity: string;
  pricePaise: string;
  priceFormatted: string;
  productDetails?: Array<{
    productId?: string;
    name?: string;
    formatted?: string;
    images?: Array<{ url?: string | null; thumbnailUrl?: string | null }>;
  }>;
};

type OrderRow = {
  orderId: string;
  userId: string;
  orderDate: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  refundSettlementStatus?: string | null;
  orderDetails?: OrderDetailRow[];
};

type PaymentIntentRow = {
  intentId: string;
  amountPaise: string;
  currency?: string | null;
  status: string;
  razorpayPaymentId?: string | null;
  createdAt: string;
};

type ShipmentRow = {
  shipmentId: string;
  status: string;
  carrier?: string | null;
  awbCode?: string | null;
  createdAt: string;
  deliveredAt?: string | null;
  trackingEventsJson?: string | null;
  shiprocketStatusId?: string | null;
  shiprocketStatusLabel?: string | null;
};

type OrderEventRow = {
  eventId: string;
  eventType: string;
  fromStatus: string;
  toStatus: string;
  actorType: string;
  message: string;
  createdAt: string;
};

type AccountOrderDetailResponse = {
  order: OrderRow;
  statusName: string;
  refundSettlementStatus?: string | null;
  paymentIntents: PaymentIntentRow[];
  shipments: ShipmentRow[];
  events: OrderEventRow[];
  fulfillmentState: string;
  paymentState: string;
};

const ORDER_DETAIL_QUERY = `query AccountOrderDetail($search: SearchOrder!) {
  searchOrder(search: $search) {
    orderId
    userId
    orderDate
    totalAmountPaise
    totalAmountFormatted
    statusId
    refundSettlementStatus
    orderDetails {
      orderDetailId
      variantId
      quantity
      pricePaise
      priceFormatted
      productDetails {
        productId
        name
        formatted
        images {
          url
          thumbnailUrl
        }
      }
    }
  }
}`;

const ORDER_STATUS_QUERY = `query AccountOrderStatuses {
  searchOrderStatus {
    statusId
    statusName
  }
}`;

const PAYMENT_QUERY = `query AccountOrderPayments($input: GetPaymentIntent!) {
  getPaymentIntent(input: $input) {
    intentId
    amountPaise
    currency
    status
    razorpayPaymentId
    createdAt
  }
}`;

const SHIPMENT_QUERY = `query AccountOrderShipments($input: GetShipment!) {
  getShipment(input: $input) {
    shipmentId
    status
    carrier
    awbCode
    createdAt
    deliveredAt
    trackingEventsJson
    shiprocketStatusId
    shiprocketStatusLabel
  }
}`;

const EVENTS_QUERY = `query AccountOrderEvents($orderId: String!) {
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

const SYNC_SHIPMENTS_MUTATION = `mutation AccountSyncOrderShipments($orderId: String!) {
  syncOrderShipmentsFromShiprocket(orderId: $orderId) {
    shipmentId
  }
}`;

function formatOrderStatusName(statusName: string): string {
  return statusName.trim().toLowerCase() === "processing" ? "processing order" : statusName;
}

function derivePaymentState(intents: PaymentIntentRow[]): string {
  if (!intents.length) return "not_started";
  const statuses = intents.map((i) => i.status.toLowerCase());
  if (statuses.some((s) => s.includes("needs_review"))) return "needs_review";
  if (statuses.some((s) => s.includes("refunded"))) return "refunded";
  if (statuses.some((s) => s.includes("paid") || s.includes("captured"))) return "paid";
  if (statuses.some((s) => s.includes("failed"))) return "failed";
  if (statuses.some((s) => s.includes("verified"))) return "verified";
  return statuses[0] ?? "pending";
}

function deriveFulfillmentState(shipments: ShipmentRow[]): string {
  if (!shipments.length) return "not_shipped";
  const fromShiprocket = (s: ShipmentRow) => {
    const id = s.shiprocketStatusId?.trim();
    if (id === "7" || id === "23") return "delivered";
    if (id === "8") return "issue";
    if (id === "9" || id === "10" || id === "14" || id === "15" || id === "16")
      return "issue";
    if (id === "17" || id === "38" || id === "56") return "out_for_delivery";
    if (
      id === "18" ||
      id === "6" ||
      id === "41" ||
      id === "45" ||
      id === "42"
    )
      return "in_transit";
    return null;
  };
  for (const s of shipments) {
    if (fromShiprocket(s) === "delivered") return "delivered";
  }
  let best: string | null = null;
  for (const s of shipments) {
    const sr = fromShiprocket(s);
    if (!sr) continue;
    if (sr === "issue") {
      best = "issue";
      break;
    }
    if (sr === "out_for_delivery") best = best ?? "out_for_delivery";
    else if (sr === "in_transit") best = best ?? "in_transit";
  }
  if (best) return best;
  const statuses = shipments.map((s) => s.status.toLowerCase());
  const labels = shipments
    .map((s) => s.shiprocketStatusLabel?.toLowerCase() ?? "")
    .filter(Boolean);
  if (
    statuses.some((x) => x.includes("delivered")) ||
    labels.some((x) => x.includes("delivered"))
  )
    return "delivered";
  if (
    statuses.some(
      (x) =>
        x.includes("failed") ||
        x.includes("returned") ||
        x.includes("rto") ||
        x.includes("cancelled")
    ) ||
    labels.some((x) => x.includes("rto") || x.includes("cancel"))
  )
    return "issue";
  if (
    statuses.some(
      (x) =>
        x.includes("shipped") ||
        x.includes("in_transit") ||
        x.includes("picked_up") ||
        x.includes("out_for_delivery") ||
        x.includes("awb_assigned")
    ) ||
    labels.some(
      (x) =>
        x.includes("transit") ||
        x.includes("picked") ||
        x.includes("delivery") ||
        x.includes("awb")
    )
  )
    return "in_transit";
  return statuses[0] ?? "pending";
}

export async function GET(
  _request: Request,
  context: { params: Promise<{ orderId: string }> }
) {
  flowLog("order detail request received");
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    flowLog("request rejected: unauthenticated");
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const { orderId } = await context.params;
  const trimmedOrderId = orderId.trim();
  if (!trimmedOrderId) {
    flowLog("request rejected: missing order id", { userId });
    return apiError("Order ID is required", 400, "VALIDATION_ERROR");
  }
  flowLog("loading order detail", { userId, orderId: trimmedOrderId });

  // Best-effort freshness: refresh courier scans from Shiprocket when available.
  // Ignore sync failures so normal order details still load.
  await callGraphqlAsCustomer(userId, SYNC_SHIPMENTS_MUTATION, {
    orderId: trimmedOrderId,
  })
    .then(() => {
      flowLog("shiprocket sync attempted", { orderId: trimmedOrderId });
    })
    .catch((e) => {
      flowLog("shiprocket sync skipped/failure (non-fatal)", {
        orderId: trimmedOrderId,
        error: e instanceof Error ? e.message : String(e),
      });
      return null;
    });

  const [orderResult, statusesResult, paymentResult, shipmentResult, eventsResult] =
    await Promise.all([
      callGraphqlAsCustomer<{ searchOrder?: OrderRow[] }>(userId, ORDER_DETAIL_QUERY, {
        search: { userId, orderId: trimmedOrderId, limit: "1", offset: "0" },
      }),
      callGraphqlAsCustomer<{
        searchOrderStatus?: Array<{ statusId: string; statusName: string }>;
      }>(userId, ORDER_STATUS_QUERY),
      callGraphqlAsCustomer<{ getPaymentIntent?: PaymentIntentRow[] }>(userId, PAYMENT_QUERY, {
        input: { orderId: trimmedOrderId },
      }),
      callGraphqlAsCustomer<{ getShipment?: ShipmentRow[] }>(userId, SHIPMENT_QUERY, {
        input: { orderId: trimmedOrderId },
      }),
      callGraphqlAsCustomer<{ getOrderEvents?: OrderEventRow[] }>(userId, EVENTS_QUERY, {
        orderId: trimmedOrderId,
      }),
    ]);

  const firstError =
    orderResult.errors?.[0]?.message ??
    statusesResult.errors?.[0]?.message ??
    paymentResult.errors?.[0]?.message ??
    shipmentResult.errors?.[0]?.message ??
    eventsResult.errors?.[0]?.message;
  if (firstError) {
    flowLog("graphql error while loading order detail", {
      orderId: trimmedOrderId,
      error: firstError,
    });
    return apiError(firstError, 400, "GRAPHQL_ERROR");
  }

  const order = orderResult.data?.searchOrder?.[0];
  if (!order) {
    flowLog("order not found", { orderId: trimmedOrderId, userId });
    return apiError("Order not found", 404, "NOT_FOUND");
  }
  if (order.userId !== userId) {
    flowLog("order identity mismatch", {
      orderId: trimmedOrderId,
      userId,
      ownerUserId: order.userId,
    });
    return apiError(
      "Order identity mismatch for authenticated customer",
      403,
      "FORBIDDEN"
    );
  }

  const statusNameById = new Map(
    (statusesResult.data?.searchOrderStatus ?? []).map((s) => [
      s.statusId,
      formatOrderStatusName(s.statusName),
    ])
  );
  const statusName = statusNameById.get(order.statusId) ?? order.statusId;
  const paymentIntents = paymentResult.data?.getPaymentIntent ?? [];
  const shipments = shipmentResult.data?.getShipment ?? [];
  const events = eventsResult.data?.getOrderEvents ?? [];

  const refundSettlementStatus =
    order.refundSettlementStatus?.trim() || null;
  const payload: AccountOrderDetailResponse = {
    order,
    statusName,
    refundSettlementStatus,
    paymentIntents,
    shipments,
    events,
    paymentState: derivePaymentState(paymentIntents),
    fulfillmentState: deriveFulfillmentState(shipments),
  };
  const eventTypes = events.map((e) => (e.eventType ?? "").trim().toLowerCase());
  flowLog("order detail loaded", {
    orderId: order.orderId,
    userId,
    statusName,
    paymentState: payload.paymentState,
    fulfillmentState: payload.fulfillmentState,
    shipmentCount: shipments.length,
    shipmentStatusIds: shipments.map((s) => s.shiprocketStatusId ?? ""),
    shipmentStatusLabels: shipments.map((s) => s.shiprocketStatusLabel ?? ""),
    refundInitiated: eventTypes.includes("refund_initiated"),
    refundProcessed: eventTypes.includes("refund_recorded"),
    refundFailed: eventTypes.includes("refund_failed"),
  });

  return Response.json({
    ok: true,
    data: payload,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
