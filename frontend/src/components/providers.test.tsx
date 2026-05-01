import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Providers } from "@/components/providers";

const usePathnameMock = vi.fn();
const ensureGuestSessionMock = vi.fn();

vi.mock("next-auth/react", () => ({
  SessionProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
}));

vi.mock("next/navigation", () => ({
  usePathname: () => usePathnameMock(),
}));

vi.mock("@/lib/session", () => ({
  ensureGuestSession: () => ensureGuestSessionMock(),
}));

vi.mock("@/components/storefront-auth-sync", () => ({
  StorefrontAuthSync: () => <div data-testid="storefront-auth-sync" />,
}));

vi.mock("@/context/storefront-login-context", () => ({
  StorefrontLoginProvider: ({ children }: { children: ReactNode }) => (
    <div data-testid="storefront-login-provider">{children}</div>
  ),
}));

describe("Providers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    usePathnameMock.mockReturnValue("/");
  });

  it("keeps storefront providers active on non-admin routes", async () => {
    render(
      <Providers>
        <div>child</div>
      </Providers>
    );

    await waitFor(() => expect(ensureGuestSessionMock).toHaveBeenCalledTimes(1));
    expect(screen.getByTestId("storefront-auth-sync")).toBeInTheDocument();
    expect(screen.getByTestId("storefront-login-provider")).toBeInTheDocument();
  });

  it("skips storefront providers on admin routes", async () => {
    usePathnameMock.mockReturnValue("/imtheboss/orders");

    render(
      <Providers>
        <div>child</div>
      </Providers>
    );

    await waitFor(() => expect(screen.queryByTestId("storefront-auth-sync")).not.toBeInTheDocument());
    expect(ensureGuestSessionMock).not.toHaveBeenCalled();
    expect(screen.queryByTestId("storefront-login-provider")).not.toBeInTheDocument();
  });
});
