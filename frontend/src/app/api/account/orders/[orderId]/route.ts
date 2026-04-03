import {
  apiError,
  callGraphql,
  requireAuthenticatedCustomerUserId,
  requireSessionToken,
} from "@/lib/server-session-auth";

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
  }>;
};

type OrderRow = {
  orderId: string;
  userId: string;
  orderDate: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
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
  const statuses = shipments.map((s) => s.status.toLowerCase());
  if (statuses.some((s) => s.includes("delivered"))) return "delivered";
  if (statuses.some((s) => s.includes("failed") || s.includes("returned"))) return "issue";
  if (statuses.some((s) => s.includes("shipped") || s.includes("in_transit"))) return "in_transit";
  return statuses[0] ?? "pending";
}

export async function GET(
  _request: Request,
  context: { params: Promise<{ orderId: string }> }
) {
  const token = await requireSessionToken();
  if (!token) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const { orderId } = await context.params;
  const trimmedOrderId = orderId.trim();
  if (!trimmedOrderId) {
    return apiError("Order ID is required", 400, "VALIDATION_ERROR");
  }

  const [orderResult, statusesResult, paymentResult, shipmentResult, eventsResult] =
    await Promise.all([
      callGraphql<{ searchOrder?: OrderRow[] }>(token, ORDER_DETAIL_QUERY, {
        search: { userId, orderId: trimmedOrderId, limit: "1", offset: "0" },
      }),
      callGraphql<{
        searchOrderStatus?: Array<{ statusId: string; statusName: string }>;
      }>(token, ORDER_STATUS_QUERY),
      callGraphql<{ getPaymentIntent?: PaymentIntentRow[] }>(token, PAYMENT_QUERY, {
        input: { orderId: trimmedOrderId },
      }),
      callGraphql<{ getShipment?: ShipmentRow[] }>(token, SHIPMENT_QUERY, {
        input: { orderId: trimmedOrderId },
      }),
      callGraphql<{ getOrderEvents?: OrderEventRow[] }>(token, EVENTS_QUERY, {
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
    return apiError(firstError, 400, "GRAPHQL_ERROR");
  }

  const order = orderResult.data?.searchOrder?.[0];
  if (!order) return apiError("Order not found", 404, "NOT_FOUND");
  if (order.userId !== userId) {
    return apiError(
      "Order identity mismatch for authenticated customer",
      403,
      "FORBIDDEN"
    );
  }

  const statusNameById = new Map(
    (statusesResult.data?.searchOrderStatus ?? []).map((s) => [s.statusId, s.statusName])
  );
  const statusName = statusNameById.get(order.statusId) ?? order.statusId;
  const paymentIntents = paymentResult.data?.getPaymentIntent ?? [];
  const shipments = shipmentResult.data?.getShipment ?? [];
  const events = eventsResult.data?.getOrderEvents ?? [];

  const payload: AccountOrderDetailResponse = {
    order,
    statusName,
    paymentIntents,
    shipments,
    events,
    paymentState: derivePaymentState(paymentIntents),
    fulfillmentState: deriveFulfillmentState(shipments),
  };

  return Response.json({
    ok: true,
    data: payload,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
