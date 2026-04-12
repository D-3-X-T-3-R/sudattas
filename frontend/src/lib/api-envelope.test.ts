import { beforeEach, describe, expect, it, vi } from "vitest";

const trackClientTelemetryMock = vi.fn();
const classifyStatusErrorMock = vi.fn(() => "recoverable");

vi.mock("@/lib/client-telemetry", () => ({
  trackClientTelemetry: (...args: unknown[]) => trackClientTelemetryMock(...args),
  classifyStatusError: (...args: unknown[]) => classifyStatusErrorMock(...args),
}));

describe("fetchApiEnvelope", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    localStorage.clear();
  });

  it("returns parsed envelope data and forwards guest session header", async () => {
    localStorage.setItem("sudattas_guest_session", "guest-1");
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({ ok: true, data: { value: 42 }, errorCode: null, message: null, fieldErrors: null, retryable: false }),
        { status: 200, headers: { "Content-Type": "application/json" } }
      )
    );
    vi.stubGlobal("fetch", fetchMock);

    const { fetchApiEnvelope } = await import("@/lib/api-envelope");
    const data = await fetchApiEnvelope<{ value: number }>("/api/products", { method: "GET" });

    expect(data).toEqual({ value: 42 });
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    const headers = new Headers(init.headers);
    expect(headers.get("X-Guest-Session-Id")).toBe("guest-1");
    expect(headers.get("X-Request-Id")).toBeTruthy();
  });

  it("refreshes browser session once on 401 then retries original request", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({ ok: false, data: null, errorCode: "UNAUTHORIZED", message: "unauthorized", fieldErrors: null, retryable: false }),
          { status: 401, headers: { "Content-Type": "application/json" } }
        )
      )
      .mockResolvedValueOnce(new Response("{}", { status: 200, headers: { "Content-Type": "application/json" } }))
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({ ok: true, data: { ok: true }, errorCode: null, message: null, fieldErrors: null, retryable: false }),
          { status: 200, headers: { "Content-Type": "application/json" } }
        )
      );
    vi.stubGlobal("fetch", fetchMock);

    const { fetchApiEnvelope } = await import("@/lib/api-envelope");
    const data = await fetchApiEnvelope<{ ok: boolean }>("/api/account/profile", { method: "GET" });

    expect(data).toEqual({ ok: true });
    expect(fetchMock).toHaveBeenCalledTimes(3);
    expect((fetchMock.mock.calls[1] as [string])[0]).toBe("/api/auth/session");
    expect((fetchMock.mock.calls[2] as [string])[0]).toBe("/api/account/profile");
  });

  it("throws ApiEnvelopeError when response is non-JSON", async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response("not-json", { status: 500 }));
    vi.stubGlobal("fetch", fetchMock);

    const { ApiEnvelopeError, fetchApiEnvelope } = await import("@/lib/api-envelope");

    await expect(fetchApiEnvelope("/api/checkout/place-order", { method: "POST" })).rejects.toBeInstanceOf(ApiEnvelopeError);
    expect(trackClientTelemetryMock).toHaveBeenCalled();
  });
});
