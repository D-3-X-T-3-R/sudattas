import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ProfilePage from "@/app/profile/page";
import { LiveAnnouncerProvider } from "@/components/ui/live-announcer";

const useSessionMock = vi.fn();
const fetchApiEnvelopeMock = vi.fn();

vi.mock("next-auth/react", () => ({
  useSession: () => useSessionMock(),
  signOut: vi.fn(),
}));

vi.mock("@/components/site-header", () => ({
  SiteHeader: () => <div>SiteHeader</div>,
}));

vi.mock("@/context/storefront-login-context", () => ({
  useStorefrontLogin: () => ({ openLogin: vi.fn() }),
}));

vi.mock("@/lib/api-envelope", () => ({
  fetchApiEnvelope: (...args: unknown[]) => fetchApiEnvelopeMock(...args),
}));

describe("ProfilePage", () => {
  beforeEach(() => {
    fetchApiEnvelopeMock.mockReset();
    useSessionMock.mockReturnValue({
      status: "authenticated",
      data: { user: { name: "Test User", email: "test@example.com" } },
    });
  });

  it("submits address form and persists via /api/account/addresses", async () => {
    const user = userEvent.setup();
    fetchApiEnvelopeMock
      .mockResolvedValueOnce({
        userId: "1",
        email: "test@example.com",
        fullName: "Test User",
        createDate: "2026-01-01T00:00:00Z",
      })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce({ shippingAddressId: "addr-1" })
      .mockResolvedValueOnce({
        userId: "1",
        email: "test@example.com",
        fullName: "Test User",
        createDate: "2026-01-01T00:00:00Z",
      })
      .mockResolvedValueOnce([
        {
          shippingAddressId: "addr-1",
          country: "India",
          stateRegion: "Karnataka",
          city: "Bengaluru",
          postalCode: "560001",
          road: "MG Road",
          apartmentNoOrName: "Test Apt",
        },
      ])
      .mockResolvedValueOnce([]);

    render(
      <LiveAnnouncerProvider>
        <ProfilePage />
      </LiveAnnouncerProvider>
    );

    await waitFor(() => {
      expect(screen.getByText("Test User")).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: /^addresses$/i }));

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /saved addresses/i })).toBeInTheDocument();
    });

    await user.type(screen.getByLabelText("Road / street"), "MG Road");
    await user.type(screen.getByLabelText("Apartment / house (optional)"), "Test Apt");
    await user.type(screen.getByLabelText("City"), "Bengaluru");
    await user.type(screen.getByLabelText("State / region"), "Karnataka");
    await user.clear(screen.getByLabelText("Country"));
    await user.type(screen.getByLabelText("Country"), "India");
    await user.type(screen.getByLabelText("Pincode"), "560001");
    await user.click(screen.getByRole("button", { name: "Save Address" }));

    await waitFor(() =>
      expect(fetchApiEnvelopeMock).toHaveBeenCalledWith(
        "/api/account/addresses",
        expect.objectContaining({ method: "POST" })
      )
    );
  });
});
