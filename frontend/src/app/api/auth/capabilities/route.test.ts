import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getAuthenticatedSession: vi.fn(),
  getAdminSession: vi.fn(),
}));

vi.mock("server-only", () => ({}));

vi.mock("@/lib/admin-auth-server", () => ({
  getAuthenticatedSession: mocks.getAuthenticatedSession,
  getAdminSession: mocks.getAdminSession,
}));

import { GET } from "@/app/api/auth/capabilities/route";

describe("GET /api/auth/capabilities", () => {
  beforeEach(() => {
    mocks.getAuthenticatedSession.mockReset();
    mocks.getAdminSession.mockReset();
  });

  it("returns guest capabilities when session lookup throws", async () => {
    mocks.getAuthenticatedSession.mockResolvedValue(null);
    mocks.getAdminSession.mockResolvedValue(null);

    const res = await GET();
    const json = (await res.json()) as {
      ok: boolean;
      data: { authenticated: boolean; admin: boolean; mode: string };
      errorCode: string | null;
    };

    expect(res.status).toBe(200);
    expect(json.ok).toBe(true);
    expect(json.errorCode).toBeNull();
    expect(json.data.authenticated).toBe(false);
    expect(json.data.admin).toBe(false);
    expect(json.data.mode).toBe("guest");
  });

  it("returns admin mode for backend-authorized admin sessions", async () => {
    mocks.getAuthenticatedSession.mockResolvedValue({
      user: { email: "admin@example.com" },
    });
    mocks.getAdminSession.mockResolvedValue({
      user: { email: "admin@example.com" },
    });

    const res = await GET();
    const json = (await res.json()) as {
      data: { authenticated: boolean; admin: boolean; mode: string };
    };

    expect(res.status).toBe(200);
    expect(json.data.authenticated).toBe(true);
    expect(json.data.admin).toBe(true);
    expect(json.data.mode).toBe("admin");
  });
});
