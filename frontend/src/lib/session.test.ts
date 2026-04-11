import { beforeEach, describe, expect, it, vi } from "vitest";

const fetchWithResilienceMock = vi.fn();
const normalizeNetworkErrorMock = vi.fn((e: unknown) => String(e));
const publicGraphqlUrlMock = vi.fn(() => "http://127.0.0.1:8080/v2");

vi.mock("@/lib/network-resilience", () => ({
  fetchWithResilience: (...args: unknown[]) => fetchWithResilienceMock(...args),
  normalizeNetworkError: (...args: unknown[]) => normalizeNetworkErrorMock(...args),
}));

vi.mock("@/lib/env/public", () => ({
  publicGraphqlUrl: () => publicGraphqlUrlMock(),
}));

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("session helpers", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    localStorage.clear();
  });

  it("returns existing guest session from localStorage without network", async () => {
    localStorage.setItem("sudattas_guest_session", "guest-existing");

    const { ensureGuestSession } = await import("@/lib/session");
    const id = await ensureGuestSession();

    expect(id).toBe("guest-existing");
    expect(fetchWithResilienceMock).not.toHaveBeenCalled();
  });

  it("deduplicates concurrent ensureGuestSession requests (single-flight)", async () => {
    const gate = deferred<Response>();
    fetchWithResilienceMock.mockReturnValueOnce(gate.promise);

    const { ensureGuestSession } = await import("@/lib/session");

    const p1 = ensureGuestSession();
    const p2 = ensureGuestSession();

    expect(fetchWithResilienceMock).toHaveBeenCalledTimes(1);

    gate.resolve(
      new Response(JSON.stringify({ session_id: "guest-abc" }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      })
    );

    const [id1, id2] = await Promise.all([p1, p2]);
    expect(id1).toBe("guest-abc");
    expect(id2).toBe("guest-abc");
    expect(localStorage.getItem("sudattas_guest_session")).toBe("guest-abc");
  });

  it("returns null when guest session endpoint fails", async () => {
    fetchWithResilienceMock.mockResolvedValueOnce(
      new Response(JSON.stringify({ error: "redis unavailable" }), {
        status: 503,
        headers: { "Content-Type": "application/json" },
      })
    );

    const { ensureGuestSession } = await import("@/lib/session");
    const id = await ensureGuestSession();

    expect(id).toBeNull();
    expect(localStorage.getItem("sudattas_guest_session")).toBeNull();
    expect(fetchWithResilienceMock).toHaveBeenCalledTimes(1);
  });
});
