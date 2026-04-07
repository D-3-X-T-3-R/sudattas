import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getServerSession: vi.fn(),
  callGraphql: vi.fn(),
}));

vi.mock("server-only", () => ({}));

vi.mock("next-auth", () => ({
  getServerSession: mocks.getServerSession,
}));

vi.mock("@/lib/auth", () => ({
  authOptions: {},
}));

vi.mock("@/lib/server-session-auth", () => ({
  callGraphql: (...args: unknown[]) => mocks.callGraphql(...args),
}));

import { getAdminSession, getAuthenticatedSession } from "@/lib/admin-auth-server";

describe("admin-auth-server", () => {
  beforeEach(() => {
    mocks.getServerSession.mockReset();
    mocks.callGraphql.mockReset();
  });

  it("returns null when session resolution throws", async () => {
    mocks.getServerSession.mockRejectedValue(new Error("session unavailable"));

    await expect(getAdminSession()).resolves.toBeNull();
  });

  it("returns authenticated session when session resolution works", async () => {
    mocks.getServerSession.mockResolvedValue({
      user: { email: "admin@example.com" },
      idToken: "id.token.value",
    });
    await expect(getAuthenticatedSession()).resolves.toMatchObject({
      user: { email: "admin@example.com" },
    });
  });

  it("returns admin session only when backend probe succeeds", async () => {
    mocks.getServerSession.mockResolvedValue({
      user: { email: "admin@example.com" },
      idToken: "id.token.value",
    });
    mocks.callGraphql.mockResolvedValue({ data: { searchUser: [{ userId: "1" }] } });

    await expect(getAdminSession()).resolves.toMatchObject({
      user: { email: "admin@example.com" },
    });
  });

  it("returns null admin session when backend probe reports auth error", async () => {
    mocks.getServerSession.mockResolvedValue({
      user: { email: "user@example.com" },
      idToken: "id.token.value",
    });
    mocks.callGraphql.mockResolvedValue({
      errors: [{ message: "Admin authorization required" }],
    });

    await expect(getAdminSession()).resolves.toBeNull();
  });
});
