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

import { GET } from "@/app/api/account/orders/[orderId]/invoice/route";

describe("GET /api/account/orders/[orderId]/invoice", () => {
  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
  });

  it("returns unauthorized when canonical user id is missing", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const res = await GET(new Request("http://localhost/api/account/orders/7/invoice"), {
      params: Promise.resolve({ orderId: "7" }),
    });
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(401);
    expect(json.errorCode).toBe("UNAUTHORIZED");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("returns validation error when order id is empty", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");

    const res = await GET(new Request("http://localhost/api/account/orders/ /invoice"), {
      params: Promise.resolve({ orderId: "   " }),
    });
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(400);
    expect(json.errorCode).toBe("VALIDATION_ERROR");
  });

  it("returns pdf payload when invoice is available", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      data: {
        getOrderInvoiceDownload: {
          invoiceId: "71",
          invoiceNumber: "INV-20260426-000071",
          orderId: "7",
          generatedAt: "2026-04-26T12:00:00Z",
          fileName: "INV-20260426-000071.pdf",
          contentType: "application/pdf",
          pdfBase64: Buffer.from("%PDF-1.4 test").toString("base64"),
        },
      },
    });

    const res = await GET(new Request("http://localhost/api/account/orders/7/invoice"), {
      params: Promise.resolve({ orderId: " 7 " }),
    });

    expect(res.status).toBe(200);
    expect(res.headers.get("content-type")).toBe("application/pdf");
    expect(res.headers.get("content-disposition")).toContain("INV-20260426-000071.pdf");
    const body = Buffer.from(await res.arrayBuffer()).toString("utf8");
    expect(body).toContain("%PDF-1.4 test");

    expect(mocks.callGraphqlAsCustomer).toHaveBeenCalledTimes(1);
    const [, , variables] = mocks.callGraphqlAsCustomer.mock.calls[0];
    expect(variables).toEqual({ orderId: "7" });
  });

  it("maps missing invoice to 404", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      errors: [{ message: "invoice not found" }],
    });

    const res = await GET(new Request("http://localhost/api/account/orders/404/invoice"), {
      params: Promise.resolve({ orderId: "404" }),
    });
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(404);
    expect(json.errorCode).toBe("NOT_FOUND");
  });

  it("maps access denied to 403", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer.mockResolvedValue({
      errors: [{ message: "Order not found for current user" }],
    });

    const res = await GET(new Request("http://localhost/api/account/orders/22/invoice"), {
      params: Promise.resolve({ orderId: "22" }),
    });
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(403);
    expect(json.errorCode).toBe("FORBIDDEN");
  });
});
