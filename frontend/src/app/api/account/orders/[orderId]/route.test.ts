import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  requireAuthenticatedCustomerUserId: vi.fn<() => Promise<string | null>>(),
  callGraphqlAsCustomer: vi.fn(),
}));

vi.mock("@/lib/server-session-auth", () => ({
  apiError: (message: string, status: number, errorCode: string) =>
    Response.json(
      {
        ok: false,
        data: null,
        errorCode,
        message,
        fieldErrors: null,
        retryable: status >= 500,
      },
      { status }
    ),
  requireAuthenticatedCustomerUserId: mocks.requireAuthenticatedCustomerUserId,
  callGraphqlAsCustomer: mocks.callGraphqlAsCustomer,
}));

import { GET } from "@/app/api/account/orders/[orderId]/route";

describe("GET /api/account/orders/[orderId]", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("returns unauthorized when canonical user id is missing", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const res = await GET(new Request("http://localhost/api/account/orders/7"), {
      params: Promise.resolve({ orderId: "7" }),
    });
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(401);
    expect(json.errorCode).toBe("UNAUTHORIZED");
  });

  it("derives cancelled fulfillment and refund flags from shipment/events", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({ data: { syncOrderShipmentsFromShiprocket: [] } }) // best-effort sync
      .mockResolvedValueOnce({
        data: {
          searchOrder: [
            {
              orderId: "7",
              userId: "104",
              orderDate: "2026-04-10T00:00:00Z",
              cancelWindowEndsAt: "2026-04-10T12:00:00Z",
              earliestBookingAt: "2026-04-10T12:00:00Z",
              pickupTargetAt: "2026-04-12T00:00:00Z",
              fulfillmentStatus: "not_created",
              totalAmountPaise: "90499",
              totalAmountFormatted: "Rs 904.99",
              statusId: "6",
              refundSettlementStatus: "refund_pending",
              orderDetails: [],
            },
          ],
        },
      })
      .mockResolvedValueOnce({
        data: { searchOrderStatus: [{ statusId: "6", statusName: "cancelled" }] },
      })
      .mockResolvedValueOnce({
        data: {
          getPaymentIntent: [
            {
              intentId: "1",
              amountPaise: "90499",
              currency: "INR",
              status: "processed",
              razorpayPaymentId: "pay_test",
              createdAt: "2026-04-10T00:00:00Z",
            },
          ],
        },
      })
      .mockResolvedValueOnce({
        data: {
          getOrderEvents: [
            {
              eventId: "e1",
              eventType: "refund_initiated",
              fromStatus: "cancelled",
              toStatus: "cancelled",
              actorType: "system",
              message: "Refund initiated",
              createdAt: "2026-04-10T00:00:00Z",
            },
          ],
        },
      })
      .mockResolvedValueOnce({
        data: {
          getRefunds: [],
        },
      })
      .mockResolvedValueOnce({
        data: {
          searchReturnRequests: [],
        },
      })
      .mockResolvedValueOnce({
        data: {
          getShipment: [
            {
              shipmentId: "99",
              status: "failed",
              carrier: "Test",
              awbCode: "AWB123",
              createdAt: "2026-04-10T00:00:00Z",
              deliveredAt: null,
              trackingEventsJson: null,
              shiprocketStatusId: "8",
              shiprocketStatusLabel: "CANCELLED",
            },
          ],
        },
      });

    const res = await GET(new Request("http://localhost/api/account/orders/7"), {
      params: Promise.resolve({ orderId: "7" }),
    });
    const json = (await res.json()) as {
      ok: boolean;
      data: {
        order: {
          cancelWindowEndsAt?: string | null;
          earliestBookingAt?: string | null;
          pickupTargetAt?: string | null;
          fulfillmentStatus?: string | null;
          refundSettlementStatus?: string | null;
        };
        statusName: string;
        fulfillmentState: string;
        paymentState: string;
        shipments: Array<{ shiprocketStatusId?: string; shiprocketStatusLabel?: string }>;
        events: Array<{ eventType: string }>;
      };
    };

    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.data.statusName).toBe("cancelled");
    expect(json.data.fulfillmentState).toBe("issue");
    expect(json.data.order.cancelWindowEndsAt).toBe("2026-04-10T12:00:00Z");
    expect(json.data.order.earliestBookingAt).toBe("2026-04-10T12:00:00Z");
    expect(json.data.order.pickupTargetAt).toBe("2026-04-12T00:00:00Z");
    expect(json.data.order.fulfillmentStatus).toBe("not_created");
    expect(json.data.order.refundSettlementStatus).toBe("refund_pending");
    expect(json.data.paymentState).toBe("paid");
    expect(json.data.shipments[0]?.shiprocketStatusId).toBe("8");
    expect(json.data.events.some((e) => e.eventType === "refund_initiated")).toBe(true);
  });
});
