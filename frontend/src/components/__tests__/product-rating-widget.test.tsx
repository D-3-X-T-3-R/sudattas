import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ProductRatingWidget } from "@/components/product-rating-widget";
import { ApiEnvelopeError } from "@/lib/api-envelope";

const sessionMock = vi.fn();
const openLoginMock = vi.fn();
const fetchApiEnvelopeMock = vi.fn();
const refreshMock = vi.fn();

vi.mock("next-auth/react", () => ({
  useSession: () => sessionMock(),
}));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ refresh: refreshMock }),
}));

vi.mock("@/context/storefront-login-context", () => ({
  useStorefrontLogin: () => ({ openLogin: openLoginMock }),
}));

vi.mock("@/lib/api-envelope", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/lib/api-envelope")>();
  return {
    ...actual,
    fetchApiEnvelope: (...args: unknown[]) => fetchApiEnvelopeMock(...args),
  };
});

describe("ProductRatingWidget (star rating only, no review text)", () => {
  beforeEach(() => {
    sessionMock.mockReset();
    openLoginMock.mockReset();
    fetchApiEnvelopeMock.mockReset();
    refreshMock.mockReset();
  });

  it("shows the average and prompts sign-in instead of submitting when a guest clicks a star", async () => {
    sessionMock.mockReturnValue({ status: "unauthenticated", data: null });
    const user = userEvent.setup();

    render(<ProductRatingWidget productId="101" initialAverage={4} initialCount={12} />);

    expect(screen.getByText("12 ratings")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Rate 5 out of 5 stars" }));

    expect(openLoginMock).toHaveBeenCalledWith("/product/101");
    expect(fetchApiEnvelopeMock).not.toHaveBeenCalled();
  });

  it("loads the customer's existing rating when authenticated", async () => {
    sessionMock.mockReturnValue({ status: "authenticated", data: {} });
    fetchApiEnvelopeMock.mockResolvedValueOnce({ rating: 3 });

    render(<ProductRatingWidget productId="101" initialAverage={4} initialCount={12} />);

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Rate 3 out of 5 stars" })).toHaveAttribute(
        "aria-pressed",
        "true"
      )
    );
    expect(fetchApiEnvelopeMock).toHaveBeenCalledWith("/api/account/product-rating?productId=101");
  });

  it("submits a rating and refreshes to pull the real backend aggregate, without guessing a new average locally", async () => {
    sessionMock.mockReturnValue({ status: "authenticated", data: {} });
    fetchApiEnvelopeMock.mockResolvedValueOnce({ rating: null }); // initial GET: no prior rating
    fetchApiEnvelopeMock.mockResolvedValueOnce({ rating: 5 }); // POST result
    const user = userEvent.setup();

    render(<ProductRatingWidget productId="101" initialAverage={4} initialCount={1} />);

    await waitFor(() => expect(fetchApiEnvelopeMock).toHaveBeenCalledTimes(1));
    await user.click(screen.getByRole("button", { name: "Rate 5 out of 5 stars" }));

    await waitFor(() => expect(fetchApiEnvelopeMock).toHaveBeenCalledTimes(2));
    expect(fetchApiEnvelopeMock).toHaveBeenNthCalledWith(2, "/api/account/product-rating", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ productId: "101", rating: 5 }),
    });
    // No locally-guessed average — the widget doesn't have the raw sum needed to compute one
    // (the backend only ever returns the already ceil-rounded whole number), so it must ask the
    // server component to refetch the real aggregate instead.
    await waitFor(() => expect(refreshMock).toHaveBeenCalledTimes(1));
    expect(screen.getByText("1 rating")).toBeInTheDocument();
  });

  it("shows the backend's exact rejection message for an undelivered-purchase submit, not a generic retry prompt", async () => {
    sessionMock.mockReturnValue({ status: "authenticated", data: {} });
    fetchApiEnvelopeMock.mockResolvedValueOnce({ rating: null }); // initial GET: no prior rating
    fetchApiEnvelopeMock.mockRejectedValueOnce(
      new ApiEnvelopeError({
        message: "You can only review products from an order that has been delivered to you",
        status: 409,
        errorCode: "GRAPHQL_ERROR",
        fieldErrors: null,
        retryable: false,
      })
    );
    const user = userEvent.setup();

    render(<ProductRatingWidget productId="101" initialAverage={0} initialCount={0} />);
    await waitFor(() => expect(fetchApiEnvelopeMock).toHaveBeenCalledTimes(1));

    await user.click(screen.getByRole("button", { name: "Rate 5 out of 5 stars" }));

    expect(
      await screen.findByText(
        "You can only review products from an order that has been delivered to you"
      )
    ).toBeInTheDocument();
    expect(refreshMock).not.toHaveBeenCalled();
  });

  it("adopts the fresh average once the parent re-renders with new props after a refresh", async () => {
    sessionMock.mockReturnValue({ status: "unauthenticated", data: null });

    const { rerender } = render(
      <ProductRatingWidget productId="101" initialAverage={4} initialCount={1} />
    );
    expect(screen.getByText("1 rating")).toBeInTheDocument();

    // Simulates what router.refresh() causes: the server component re-fetches
    // productRatingSummary and passes the real aggregate back down as new props.
    rerender(<ProductRatingWidget productId="101" initialAverage={5} initialCount={2} />);

    expect(screen.getByText("2 ratings")).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
  });
});
