import { beforeEach, describe, expect, it, vi } from "vitest";

const fetchApiEnvelopeMock = vi.hoisted(() => vi.fn());

vi.mock("@/lib/api-envelope", () => ({
  fetchApiEnvelope: (...args: unknown[]) => fetchApiEnvelopeMock(...args),
}));

import { gqlAdmin } from "@/lib/graphqlAdmin";

describe("gqlAdmin", () => {
  beforeEach(() => {
    fetchApiEnvelopeMock.mockReset();
    fetchApiEnvelopeMock.mockResolvedValue({ searchOrder: [] });
  });

  it("routes admin order queries through /api/admin/orders", async () => {
    await gqlAdmin(`query SearchOrdersList($search: SearchOrder!) {
      searchOrder(search: $search) { orderId statusId }
    }`, { search: { userId: "", limit: "1" } });

    expect(fetchApiEnvelopeMock).toHaveBeenCalledTimes(1);
    expect(fetchApiEnvelopeMock.mock.calls[0]?.[0]).toBe("/api/admin/orders");
  });

  it("routes admin payment-intent queries through /api/admin/payment-intents", async () => {
    await gqlAdmin(`query AdminSearchPaymentIntent($input: SearchPaymentIntent!) {
      searchPaymentIntent(input: $input) { intentId status }
    }`, { input: {} });

    expect(fetchApiEnvelopeMock).toHaveBeenCalledTimes(1);
    expect(fetchApiEnvelopeMock.mock.calls[0]?.[0]).toBe("/api/admin/payment-intents");
  });

  it("routes admin exchange-request queries through /api/admin/exchanges", async () => {
    await gqlAdmin(`query AdminSearchExchangeRequests($input: SearchExchangeRequestsInput!) {
      searchExchangeRequests(input: $input) { exchangeId status }
    }`, { input: {} });

    expect(fetchApiEnvelopeMock).toHaveBeenCalledTimes(1);
    expect(fetchApiEnvelopeMock.mock.calls[0]?.[0]).toBe("/api/admin/exchanges");
  });

  it("never calls GraphQL /v2 directly from browser admin client", async () => {
    await gqlAdmin("query UnknownAdminQuery { customRoot }");

    expect(fetchApiEnvelopeMock).toHaveBeenCalledTimes(1);
    expect(fetchApiEnvelopeMock.mock.calls[0]?.[0]).toBe("/api/admin/graphql");
  });
});
