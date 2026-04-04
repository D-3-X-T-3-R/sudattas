import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useStorefrontLogin, StorefrontLoginProvider } from "@/context/storefront-login-context";
import { LiveAnnouncerProvider } from "@/components/ui/live-announcer";

const signInMock = vi.fn();

vi.mock("next-auth/react", () => ({
  signIn: (...args: unknown[]) => signInMock(...args),
}));

vi.mock("@/lib/client-telemetry", () => ({
  trackClientTelemetry: vi.fn(),
}));

function OpenDialogButton() {
  const { openLogin } = useStorefrontLogin();
  return (
    <button type="button" onClick={() => openLogin()}>
      Open Login
    </button>
  );
}

describe("StorefrontLoginProvider", () => {
  beforeEach(() => {
    signInMock.mockReset();
    vi.spyOn(window, "fetch").mockResolvedValue({
      ok: true,
      json: async () => ({ ok: true }),
    } as Response);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("supports OTP send and verify sign-in flow", async () => {
    const user = userEvent.setup();
    signInMock.mockResolvedValue({ error: "OTP verification failed" });

    render(
      <LiveAnnouncerProvider>
        <StorefrontLoginProvider>
          <OpenDialogButton />
        </StorefrontLoginProvider>
      </LiveAnnouncerProvider>
    );

    await user.click(screen.getByRole("button", { name: "Open Login" }));
    await user.type(screen.getByLabelText("Phone number"), "9876543210");
    await user.click(screen.getByRole("button", { name: "Send OTP" }));

    await waitFor(() =>
      expect(window.fetch).toHaveBeenCalledWith(
        "/api/auth/phone-otp/request",
        expect.objectContaining({ method: "POST" })
      )
    );

    await user.type(screen.getByLabelText("One-time password"), "123456");
    await user.click(screen.getByRole("button", { name: "Verify OTP and sign in" }));

    await waitFor(() =>
      expect(signInMock).toHaveBeenCalledWith(
        "phone-otp",
        expect.objectContaining({
          phone: "9876543210",
          otp: "123456",
          redirect: false,
        })
      )
    );
  });
});
