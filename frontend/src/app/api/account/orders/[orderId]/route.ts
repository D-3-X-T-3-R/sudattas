import {
  apiError,
  callGraphqlAsCustomer,
  graphqlErrorToApiStatus,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";
import {
  canonicalOrderStatusName,
  derivePaymentStateFromIntents,
  deriveShipmentState,
  statusNameFromId,
} from "@/lib/order-state";
import { formatInrFromPaise } from "@/lib/money";

type OrderDetailRow = {
  orderDetailId: string;
  variantId: string;
  quantity: string;
  pricePaise: string;
  lineTotalMinor?: string;
  itemStatus?: string;
  cancelledAt?: string | null;
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
  cancelWindowEndsAt?: string | null;
  earliestBookingAt?: string | null;
  pickupTargetAt?: string | null;
  fulfillmentStatus?: string | null;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  refundSettlementStatus?: string | null;
  paymentMethod?: string | null;
  invoiceId?: string | null;
  invoiceNumber?: string | null;
  invoiceGeneratedAt?: string | null;
  invoiceAvailable?: boolean | null;
  invoiceUrl?: string | null;
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

type RefundRow = {
  refundId: string;
  orderId: string;
  gatewayRefundId: string;
  amountPaise: string;
  currency: string;
  status: string;
  createdAt: string;
  lineItemsRefundedJson?: string | null;
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
  returnWindowDays: number;
  returnRequests: Array<{
    returnId: string;
    orderId: string;
    userId: string;
    status: string;
    reason: string;
    createdAt: string;
    receivedAt?: string | null;
    refundAttemptId?: string | null;
    items: Array<{
      returnId: string;
      orderDetailId: string;
      quantity: string;
      refundAmountMinor: string;
      status: string;
    }>;
  }>;
  refundSummary: {
    itemRefundMinor: number;
    shippingRefundMinor: number;
    totalRefundMinor: number;
    breakdownAvailable: boolean;
    totalRefundFormatted: string;
    itemRefundFormatted: string;
    shippingRefundFormatted: string;
  };
};

const ORDER_DETAIL_QUERY = `query AccountOrderDetail($search: SearchOrder!) {
  searchOrder(search: $search) {
    orderId
    userId
    orderDate
    cancelWindowEndsAt
    earliestBookingAt
    pickupTargetAt
    fulfillmentStatus
    totalAmountPaise
    totalAmountFormatted
    statusId
    refundSettlementStatus
    paymentMethod
    invoiceId
    invoiceNumber
    invoiceGeneratedAt
    invoiceAvailable
    invoiceUrl
    orderDetails {
      orderDetailId
      variantId
      quantity
      pricePaise
      lineTotalMinor
      itemStatus
      cancelledAt
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

const RETURN_REQUESTS_QUERY = `query AccountOrderReturns($input: SearchReturnRequestsInput!) {
  searchReturnRequests(input: $input) {
    returnId
    orderId
    userId
    status
    reason
    createdAt
    receivedAt
    refundAttemptId
    items {
      returnId
      orderDetailId
      quantity
      refundAmountMinor
      status
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

const REFUNDS_QUERY = `query AccountOrderRefunds($input: GetRefund!) {
  getRefunds(input: $input) {
    refundId
    orderId
    gatewayRefundId
    amountPaise
    currency
    status
    createdAt
    lineItemsRefundedJson
  }
}`;

const SYNC_SHIPMENTS_MUTATION = `mutation AccountSyncOrderShipments($orderId: String!) {
  syncOrderShipmentsFromShiprocket(orderId: $orderId) {
    shipmentId
  }
}`;

type OrderStatusesResult = {
  data?: { searchOrderStatus?: Array<{ statusId: string; statusName: string }> };
  errors?: Array<{ message?: string }>;
};

// Order statuses are global reference data, identical for every order. When a
// customer opens the Orders tab, one detail request fires per order, so
// without this cache the same status list gets refetched from the backend
// once per order in the list.
const ORDER_STATUSES_CACHE_MS = 30_000;
let cachedOrderStatuses: { result: OrderStatusesResult; expiresAt: number } | null = null;

function getOrderStatuses(userId: string): Promise<OrderStatusesResult> {
  const now = Date.now();
  if (cachedOrderStatuses && cachedOrderStatuses.expiresAt > now) {
    return Promise.resolve(cachedOrderStatuses.result);
  }
  return callGraphqlAsCustomer<{
    searchOrderStatus?: Array<{ statusId: string; statusName: string }>;
  }>(userId, ORDER_STATUS_QUERY).then((result) => {
    if (!result.errors?.length) {
      cachedOrderStatuses = { result, expiresAt: now + ORDER_STATUSES_CACHE_MS };
    }
    return result;
  });
}

function parseMinorAmount(raw: unknown): number | null {
  const value = Number(raw);
  if (!Number.isFinite(value)) return null;
  return Math.max(0, Math.trunc(value));
}

function parseRefundBreakdown(
  lineItemsRefundedJson: string | null | undefined
): { itemMinor: number; shippingMinor: number; breakdownAvailable: boolean } {
  const raw = lineItemsRefundedJson?.trim();
  if (!raw) {
    return { itemMinor: 0, shippingMinor: 0, breakdownAvailable: false };
  }
  try {
    const parsed = JSON.parse(raw) as unknown;
    const rows = Array.isArray(parsed)
      ? parsed
      : parsed &&
          typeof parsed === "object" &&
          Array.isArray((parsed as { items?: unknown[] }).items)
        ? ((parsed as { items: unknown[] }).items ?? [])
        : [];

    if (!rows.length) {
      return { itemMinor: 0, shippingMinor: 0, breakdownAvailable: false };
    }

    let itemMinor = 0;
    let shippingMinor = 0;
    let seenAmount = false;

    for (const row of rows) {
      if (!row || typeof row !== "object") continue;
      const entry = row as Record<string, unknown>;
      const amount =
        parseMinorAmount(entry.amount_paise) ??
        parseMinorAmount(entry.amountPaise) ??
        parseMinorAmount(entry.amount_minor) ??
        parseMinorAmount(entry.amountMinor);
      if (amount === null) continue;
      seenAmount = true;

      const kind = [
        entry.type,
        entry.item_type,
        entry.kind,
        entry.category,
        entry.component,
      ]
        .filter((v): v is string => typeof v === "string")
        .map((v) => v.toLowerCase());
      const isShipping =
        entry.shipping === true || kind.some((v) => v.includes("ship"));

      if (isShipping) shippingMinor += amount;
      else itemMinor += amount;
    }

    return { itemMinor, shippingMinor, breakdownAvailable: seenAmount };
  } catch {
    return { itemMinor: 0, shippingMinor: 0, breakdownAvailable: false };
  }
}

export async function GET(
  _request: Request,
  context: { params: Promise<{ orderId: string }> }
) {
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const { orderId } = await context.params;
  const trimmedOrderId = orderId.trim();
  if (!trimmedOrderId) {
    return apiError("Order ID is required", 400, "VALIDATION_ERROR");
  }

  // Best-effort freshness: refresh courier scans from Shiprocket when available.
  // Ignore sync failures so normal order details still load. Only the shipment
  // query depends on this, so run it alongside the other queries instead of
  // blocking the whole response (and the order's product images) on it.
  const shipmentSyncPromise = callGraphqlAsCustomer(userId, SYNC_SHIPMENTS_MUTATION, {
    orderId: trimmedOrderId,
  }).catch(() => null);

  const [orderResult, statusesResult, paymentResult, shipmentResult, eventsResult, refundsResult, returnsResult] =
    await Promise.all([
      callGraphqlAsCustomer<{ searchOrder?: OrderRow[] }>(userId, ORDER_DETAIL_QUERY, {
        search: { userId, orderId: trimmedOrderId, limit: "1", offset: "0" },
      }),
      getOrderStatuses(userId),
      callGraphqlAsCustomer<{ getPaymentIntent?: PaymentIntentRow[] }>(userId, PAYMENT_QUERY, {
        input: { orderId: trimmedOrderId },
      }),
      shipmentSyncPromise.then(() =>
        callGraphqlAsCustomer<{ getShipment?: ShipmentRow[] }>(userId, SHIPMENT_QUERY, {
          input: { orderId: trimmedOrderId },
        })
      ),
      callGraphqlAsCustomer<{ getOrderEvents?: OrderEventRow[] }>(userId, EVENTS_QUERY, {
        orderId: trimmedOrderId,
      }),
      callGraphqlAsCustomer<{ getRefunds?: RefundRow[] }>(userId, REFUNDS_QUERY, {
        input: { orderId: trimmedOrderId },
      }),
      callGraphqlAsCustomer<{
        searchReturnRequests?: AccountOrderDetailResponse["returnRequests"];
      }>(userId, RETURN_REQUESTS_QUERY, {
        input: { orderId: trimmedOrderId },
      }),
    ]);

  const firstErrors =
    orderResult.errors ??
    statusesResult.errors ??
    paymentResult.errors ??
    shipmentResult.errors ??
    eventsResult.errors ??
    refundsResult.errors ??
    returnsResult.errors;
  if (firstErrors?.length) {
    const { status, message } = graphqlErrorToApiStatus(
      firstErrors,
      "Failed to load order details"
    );
    return apiError(message, status, "GRAPHQL_ERROR");
  }

  const order = orderResult.data?.searchOrder?.[0];
  if (!order) {
    return apiError("Order not found", 404, "NOT_FOUND");
  }
  if (order.userId !== userId) {
    return apiError(
      "Order identity mismatch for authenticated customer",
      403,
      "FORBIDDEN"
    );
  }

  const statusNameById = new Map(
    (statusesResult.data?.searchOrderStatus ?? []).map((s) => [
      s.statusId,
      canonicalOrderStatusName(s.statusName),
    ])
  );
  const statusName = statusNameFromId(order.statusId, statusNameById);
  const paymentIntents = paymentResult.data?.getPaymentIntent ?? [];
  const shipments = shipmentResult.data?.getShipment ?? [];
  const events = eventsResult.data?.getOrderEvents ?? [];
  const refunds = refundsResult.data?.getRefunds ?? [];
  const returnRequests = returnsResult.data?.searchReturnRequests ?? [];
  const returnWindowDays = Number.parseInt(
    (process.env.RETURN_WINDOW_DAYS ?? "7").trim(),
    10
  );
  const normalizedReturnWindowDays =
    Number.isFinite(returnWindowDays) && returnWindowDays > 0
      ? returnWindowDays
      : 7;

  const refundSettlementStatus = order.refundSettlementStatus?.trim() || null;
  const processedRefunds = refunds.filter(
    (r) => (r.status ?? "").trim().toLowerCase() === "processed"
  );
  const totalRefundMinor = processedRefunds.reduce((sum, refund) => {
    const amount = parseMinorAmount(refund.amountPaise);
    return sum + (amount ?? 0);
  }, 0);

  let itemRefundMinor = totalRefundMinor;
  let shippingRefundMinor = 0;
  let breakdownAvailable = false;

  if (processedRefunds.length > 0) {
    let parsedItemMinor = 0;
    let parsedShippingMinor = 0;
    let hasParsedBreakdown = false;

    for (const refund of processedRefunds) {
      const parsed = parseRefundBreakdown(refund.lineItemsRefundedJson);
      if (!parsed.breakdownAvailable) continue;
      hasParsedBreakdown = true;
      parsedItemMinor += parsed.itemMinor;
      parsedShippingMinor += parsed.shippingMinor;
    }

    if (hasParsedBreakdown) {
      const parsedTotal = parsedItemMinor + parsedShippingMinor;
      const remainder = totalRefundMinor - parsedTotal;
      itemRefundMinor = parsedItemMinor + Math.max(0, remainder);
      shippingRefundMinor = parsedShippingMinor;
      breakdownAvailable = true;
    }
  }
  const payload: AccountOrderDetailResponse = {
    order,
    statusName,
    refundSettlementStatus,
    paymentIntents,
    shipments,
    events,
    paymentState: derivePaymentStateFromIntents(paymentIntents),
    fulfillmentState: deriveShipmentState(shipments),
    returnRequests,
    returnWindowDays: normalizedReturnWindowDays,
    refundSummary: {
      itemRefundMinor,
      shippingRefundMinor,
      totalRefundMinor,
      breakdownAvailable,
      totalRefundFormatted: formatInrFromPaise(totalRefundMinor),
      itemRefundFormatted: formatInrFromPaise(itemRefundMinor),
      shippingRefundFormatted: formatInrFromPaise(shippingRefundMinor),
    },
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
