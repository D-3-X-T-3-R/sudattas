import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getServerSession: vi.fn(),
}));

vi.mock("server-only", () => ({}));

vi.mock("next-auth", () => ({
  getServerSession: mocks.getServerSession,
}));

vi.mock("@/lib/auth", () => ({
  authOptions: {},
}));

vi.mock("@/lib/env/server", () => ({
  serverEnv: {
    ADMIN_ALLOWED_EMAILS: "admin@example.com, owner@example.com",
  },
}));

import { getAdminSession, isAdminEmail } from "@/lib/admin-auth-server";

describe("admin-auth-server", () => {
  beforeEach(() => {
    mocks.getServerSession.mockReset();
  });

  it("returns null when session resolution throws", async () => {
    mocks.getServerSession.mockRejectedValue(new Error("session unavailable"));

    await expect(getAdminSession()).resolves.toBeNull();
  });

  it("accepts emails from allowlist and rejects others", async () => {
    expect(isAdminEmail("admin@example.com")).toBe(true);
    expect(isAdminEmail("owner@example.com")).toBe(true);
    expect(isAdminEmail("user@example.com")).toBe(false);
  });
});
