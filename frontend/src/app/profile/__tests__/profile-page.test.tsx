import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ProfilePage from "@/app/profile/page";
import { LiveAnnouncerProvider } from "@/components/ui/live-announcer";

/* eslint-disable max-lines-per-function */

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

  const baseProfile = {
    userId: "1",
    email: "test@example.com",
    fullName: "Test User",
    createDate: "2026-01-01T00:00:00Z",
  };

  const baseOrder = {
    orderId: "7001",
    userId: "1",
    orderDate: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
    totalAmountPaise: "150000",
    totalAmountFormatted: "₹1500.00",
    statusId: "6",
    statusName: "delivered",
    paymentMethod: "prepaid",
    cancelWindowHours: 12,
    returnWindowDays: 7,
  };

  function makeOrderDetailPayload(overrides?: {
    paymentMethod?: string;
    deliveredAt?: string | null;
    returnWindowDays?: number;
    returnRequests?: Array<{
      returnId: string;
      orderId: string;
      userId: string;
      status: string;
      reason: string;
      createdAt: string;
      receivedAt?: string | null;
      refundAttemptId?: string | null;
      items: Array<{
        returnId: string;
        orderDetailId: string;
        quantity: string;
        refundAmountMinor: string;
        status: string;
      }>;
    }>;
  }) {
    return {
      order: {
        ...baseOrder,
        paymentMethod: overrides?.paymentMethod ?? "prepaid",
        orderDetails: [
          {
            orderDetailId: "501",
            variantId: "11",
            quantity: "1",
            pricePaise: "150000",
            lineTotalMinor: "150000",
            itemStatus: "active",
            cancelledAt: null,
            priceFormatted: "₹1500.00",
            productDetails: [
              {
                productId: "91",
                name: "Test Saree",
                formatted: "Rich weave",
                images: [],
              },
            ],
          },
        ],
      },
      statusName: "delivered",
      refundSettlementStatus: null,
      paymentIntents: [],
      shipments: [
        {
          shipmentId: "301",
          status: "delivered",
          carrier: "Test",
          awbCode: "AWB301",
          createdAt: new Date(Date.now() - 36 * 60 * 60 * 1000).toISOString(),
          deliveredAt:
            overrides?.deliveredAt ??
            new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
          trackingEventsJson: null,
          shiprocketStatusId: "7",
          shiprocketStatusLabel: "Delivered",
        },
      ],
      events: [],
      fulfillmentState: "delivered",
      paymentState: "paid",
      returnWindowDays: overrides?.returnWindowDays ?? 7,
      returnRequests: overrides?.returnRequests ?? [],
      refundSummary: {
        itemRefundMinor: 0,
        shippingRefundMinor: 0,
        totalRefundMinor: 0,
        totalRefundFormatted: "₹0.00",
        itemRefundFormatted: "₹0.00",
        shippingRefundFormatted: "₹0.00",
      },
    };
  }

  async function renderOrdersWithDetail(detailPayload: ReturnType<typeof makeOrderDetailPayload>, orderOverrides?: Partial<typeof baseOrder>) {
    const user = userEvent.setup();
    fetchApiEnvelopeMock
      .mockResolvedValueOnce(baseProfile)
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([{ ...baseOrder, ...orderOverrides }])
      .mockResolvedValueOnce(detailPayload);

    render(
      <LiveAnnouncerProvider>
        <ProfilePage />
      </LiveAnnouncerProvider>
    );

    await waitFor(() => {
      expect(screen.getByText("Test User")).toBeInTheDocument();
    });
    await user.click(screen.getByRole("button", { name: /^orders$/i }));
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /recent orders/i })).toBeInTheDocument();
    });
  }

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

    fireEvent.input(screen.getByLabelText("Road / street"), { target: { value: "MG Road" } });
    fireEvent.input(screen.getByLabelText("Apartment / house (optional)"), { target: { value: "Test Apt" } });
    fireEvent.input(screen.getByLabelText("City"), { target: { value: "Bengaluru" } });
    fireEvent.input(screen.getByLabelText("State / region"), { target: { value: "Karnataka" } });
    fireEvent.input(screen.getByLabelText("Country"), { target: { value: "India" } });
    fireEvent.input(screen.getByLabelText("Pincode"), { target: { value: "560001" } });
    await user.click(screen.getByRole("button", { name: "Save Address" }));

    await waitFor(() =>
      expect(fetchApiEnvelopeMock).toHaveBeenCalledWith(
        "/api/account/addresses",
        expect.objectContaining({ method: "POST" })
      )
    );
  });

  it("shows return selection controls only for eligible prepaid delivered orders within window", async () => {
    await renderOrdersWithDetail(makeOrderDetailPayload());
    await waitFor(() => {
      expect(screen.getByText("Select for return")).toBeInTheDocument();
      expect(screen.getByText("Request return")).toBeInTheDocument();
    });
  });

  it("does not show return selection for COD orders and shows prepaid-only copy", async () => {
    await renderOrdersWithDetail(
      makeOrderDetailPayload({ paymentMethod: "cod" }),
      { paymentMethod: "cod" }
    );
    await waitFor(() => {
      expect(screen.queryByText("Select for return")).not.toBeInTheDocument();
      expect(
        screen.getByText("Returns are available only for prepaid orders.")
      ).toBeInTheDocument();
    });
  });

  it("does not show return selection after return window closes", async () => {
    await renderOrdersWithDetail(
      makeOrderDetailPayload({
        deliveredAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
        returnWindowDays: 7,
      }),
      { returnWindowDays: 7 }
    );
    await waitFor(() => {
      expect(screen.queryByText("Select for return")).not.toBeInTheDocument();
      expect(screen.getByText("Return window closed.")).toBeInTheDocument();
    });
  });

  it("renders return/refund status labels from return request state", async () => {
    await renderOrdersWithDetail(
      makeOrderDetailPayload({
        returnRequests: [
          {
            returnId: "88",
            orderId: "7001",
            userId: "1",
            status: "refund_pending",
            reason: "Color mismatch",
            createdAt: "2026-04-24T10:00:00Z",
            items: [
              {
                returnId: "88",
                orderDetailId: "501",
                quantity: "1",
                refundAmountMinor: "150000",
                status: "refund_pending",
              },
            ],
          },
        ],
      })
    );
    await waitFor(() => {
      expect(screen.getByText("Refund processing")).toBeInTheDocument();
    });
  });
});
