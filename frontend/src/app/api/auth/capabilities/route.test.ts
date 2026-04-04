import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getServerSession: vi.fn(),
  isAdminEmail: vi.fn(),
}));

vi.mock("server-only", () => ({}));

vi.mock("next-auth", () => ({
  getServerSession: mocks.getServerSession,
}));

vi.mock("@/lib/auth", () => ({
  authOptions: {},
}));

vi.mock("@/lib/admin-auth-server", () => ({
  isAdminEmail: mocks.isAdminEmail,
}));

import { GET } from "@/app/api/auth/capabilities/route";

describe("GET /api/auth/capabilities", () => {
  beforeEach(() => {
    mocks.getServerSession.mockReset();
    mocks.isAdminEmail.mockReset();
  });

  it("returns guest capabilities when session lookup throws", async () => {
    mocks.getServerSession.mockRejectedValue(new Error("session unavailable"));
    mocks.isAdminEmail.mockReturnValue(false);

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

  it("returns admin mode for allowlisted admin sessions", async () => {
    mocks.getServerSession.mockResolvedValue({
      user: { email: "admin@example.com" },
    });
    mocks.isAdminEmail.mockReturnValue(true);

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
