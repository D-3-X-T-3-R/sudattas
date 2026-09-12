import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useStorefrontLogin, StorefrontLoginProvider } from "@/context/storefront-login-context";

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
    <button type="button" onClick={() => openLogin("/checkout")}>
      Open Login
    </button>
  );
}

describe("StorefrontLoginProvider", () => {
  beforeEach(() => {
    signInMock.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("signs in with Google, forwarding the callback URL from openLogin", async () => {
    const user = userEvent.setup();
    signInMock.mockResolvedValue(undefined);

    render(
      <StorefrontLoginProvider>
        <OpenDialogButton />
      </StorefrontLoginProvider>
    );

    await user.click(screen.getByRole("button", { name: "Open Login" }));
    await user.click(screen.getByRole("button", { name: "Continue with Google" }));

    expect(signInMock).toHaveBeenCalledWith("google", { callbackUrl: "/checkout" });
  });

  it("does not offer a phone/OTP sign-in option", async () => {
    const user = userEvent.setup();

    render(
      <StorefrontLoginProvider>
        <OpenDialogButton />
      </StorefrontLoginProvider>
    );

    await user.click(screen.getByRole("button", { name: "Open Login" }));

    expect(screen.queryByLabelText(/phone number/i)).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/one-time password/i)).not.toBeInTheDocument();
  });
});
