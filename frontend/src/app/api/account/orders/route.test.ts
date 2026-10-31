import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  requireAuthenticatedCustomerUserId: vi.fn<() => Promise<string | null>>(),
  callGraphqlAsCustomer: vi.fn(),
}));

// Mirrors the real graphqlErrorToApiStatus (src/lib/server-session-auth.ts) rather than
// partially mocking via importOriginal — the real module `import "server-only"`, which throws
// outside a Server Component context, so the actual implementation can't be pulled in here.
function mockGraphqlErrorToApiStatus(
  errors: Array<{ message?: string; extensions?: { code?: string } }> | undefined,
  fallbackMessage: string
): { status: number; message: string } {
  const firstError = errors?.[0];
  const message = firstError?.message?.trim() || fallbackMessage;
  const code = firstError?.extensions?.code;
  const lower = message.toLowerCase();
  const status =
    code === "Unauthenticated"
      ? 401
      : code === "PermissionDenied"
        ? 403
        : code === "NotFound" || lower.includes("not found")
          ? 404
          : code === "FailedPrecondition" || lower.includes("failed_precondition")
            ? 409
            : code === "Aborted"
              ? 409
              : code === "OutOfRange"
                ? 400
                : code === "Unimplemented"
                  ? 501
                  : code === "Unavailable"
                    ? 503
                    : code === "DeadlineExceeded"
                      ? 504
                      : code === "Cancelled"
                        ? 503
                        : lower.includes("illegal") || lower.includes("invalid")
                          ? 400
                          : 400;
  return { status, message };
}

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
  graphqlErrorToApiStatus: mockGraphqlErrorToApiStatus,
  requireAuthenticatedCustomerUserId: mocks.requireAuthenticatedCustomerUserId,
  callGraphqlAsCustomer: mocks.callGraphqlAsCustomer,
}));

import { GET } from "@/app/api/account/orders/route";

describe("GET /api/account/orders", () => {
  const originalCancelWindow = process.env.CANCEL_WINDOW_HOURS;

  beforeEach(() => {
    mocks.requireAuthenticatedCustomerUserId.mockReset();
    mocks.callGraphqlAsCustomer.mockReset();
    process.env.CANCEL_WINDOW_HOURS = "18";
  });

  afterEach(() => {
    process.env.CANCEL_WINDOW_HOURS = originalCancelWindow;
  });

  it("returns unauthorized when canonical user id is missing", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue(null);

    const res = await GET();
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(401);
    expect(json.errorCode).toBe("UNAUTHORIZED");
    expect(mocks.callGraphqlAsCustomer).not.toHaveBeenCalled();
  });

  it("returns backend totals/status and only adds mapped statusName + cancelWindowHours", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({
        data: {
          searchOrder: [
            {
              orderId: "67",
              userId: "104",
              orderDate: "2026-04-18T20:33:31Z",
              cancelWindowEndsAt: "2026-04-19T08:33:31Z",
              totalAmountPaise: "90735",
              totalAmountFormatted: "Rs 907.35",
              statusId: "2",
            },
          ],
        },
      })
      .mockResolvedValueOnce({
        data: {
          searchOrderStatus: [{ statusId: "2", statusName: "processing" }],
        },
      });

    const res = await GET();
    const json = (await res.json()) as {
      ok: boolean;
      data: Array<{
        orderId: string;
        userId: string;
        cancelWindowEndsAt?: string | null;
        totalAmountPaise: string;
        statusId: string;
        statusName: string;
        cancelWindowHours: number;
      }>;
    };

    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.data).toHaveLength(1);
    expect(json.data[0]).toMatchObject({
      orderId: "67",
      userId: "104",
      cancelWindowEndsAt: "2026-04-19T08:33:31Z",
      totalAmountPaise: "90735",
      statusId: "2",
      statusName: "processing order",
      cancelWindowHours: 18,
    });
  });

  it("rejects identity mismatch from backend payload", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({
        data: {
          searchOrder: [
            {
              orderId: "67",
              userId: "999",
              orderDate: "2026-04-18T20:33:31Z",
              totalAmountPaise: "90735",
              totalAmountFormatted: "Rs 907.35",
              statusId: "2",
            },
          ],
        },
      })
      .mockResolvedValueOnce({
        data: {
          searchOrderStatus: [{ statusId: "2", statusName: "processing" }],
        },
      });

    const res = await GET();
    const json = (await res.json()) as { errorCode: string };

    expect(res.status).toBe(403);
    expect(json.errorCode).toBe("FORBIDDEN");
  });

  it("maps GraphQL error to API error envelope", async () => {
    mocks.requireAuthenticatedCustomerUserId.mockResolvedValue("104");
    mocks.callGraphqlAsCustomer
      .mockResolvedValueOnce({
        errors: [{ message: "backend orders query failed" }],
      })
      .mockResolvedValueOnce({
        data: { searchOrderStatus: [] },
      });

    const res = await GET();
    const json = (await res.json()) as { errorCode: string; message: string };

    expect(res.status).toBe(400);
    expect(json.errorCode).toBe("GRAPHQL_ERROR");
    expect(json.message).toContain("backend orders query failed");
  });
});
