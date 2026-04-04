import { render, screen, waitFor } from "@testing-library/react";
import CheckoutAddressPage from "@/app/checkout/address/page";
import { LiveAnnouncerProvider } from "@/components/ui/live-announcer";

const useSessionMock = vi.fn();
const openLoginMock = vi.fn();
const replaceMock = vi.fn();
const fetchApiEnvelopeMock = vi.fn();

vi.mock("next-auth/react", () => ({
  useSession: () => useSessionMock(),
}));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace: replaceMock }),
}));

vi.mock("@/components/site-header", () => ({
  SiteHeader: () => <div>SiteHeader</div>,
}));

vi.mock("@/context/storefront-login-context", () => ({
  useStorefrontLogin: () => ({ openLogin: openLoginMock }),
}));

vi.mock("@/context/storefront-context", () => ({
  useStorefront: () => ({ cartLines: [] }),
}));

vi.mock("@/hooks/use-razorpay-test", () => ({
  useRazorpayTest: () => ({
    paymentLoading: false,
    paymentMessage: null,
    runCheckout: vi.fn(),
  }),
}));

vi.mock("@/lib/api-envelope", () => ({
  fetchApiEnvelope: (...args: unknown[]) => fetchApiEnvelopeMock(...args),
}));

describe("CheckoutAddressPage states", () => {
  beforeEach(() => {
    fetchApiEnvelopeMock.mockReset();
    replaceMock.mockReset();
    openLoginMock.mockReset();
  });

  it("renders unauthenticated sign-in state", () => {
    useSessionMock.mockReturnValue({ status: "unauthenticated", data: null });
    render(
      <LiveAnnouncerProvider>
        <CheckoutAddressPage />
      </LiveAnnouncerProvider>
    );
    expect(screen.getByText("Sign in to continue")).toBeInTheDocument();
  });

  it("renders loading state while session is loading", () => {
    useSessionMock.mockReturnValue({ status: "loading", data: null });
    render(
      <LiveAnnouncerProvider>
        <CheckoutAddressPage />
      </LiveAnnouncerProvider>
    );
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("redirects authenticated users with empty cart to /bag", async () => {
    useSessionMock.mockReturnValue({ status: "authenticated", data: { user: { email: "a@b.com" } } });
    fetchApiEnvelopeMock.mockResolvedValue([]);
    render(
      <LiveAnnouncerProvider>
        <CheckoutAddressPage />
      </LiveAnnouncerProvider>
    );

    await waitFor(() => expect(replaceMock).toHaveBeenCalledWith("/bag"));
  });
});
