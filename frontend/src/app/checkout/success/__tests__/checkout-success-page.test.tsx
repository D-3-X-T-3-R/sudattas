import { render, screen } from "@testing-library/react";
import CheckoutSuccessPage from "@/app/checkout/success/page";

describe("CheckoutSuccessPage payment-state messaging", () => {
  it("does not show verified success copy for pending payments", async () => {
    const page = await CheckoutSuccessPage({
      searchParams: Promise.resolve({ orderId: "9001", payment: "pending" }),
    });
    render(page);

    expect(screen.getByText("We're confirming your payment")).toBeInTheDocument();
    expect(
      screen.getByText("We're confirming your payment. Please don't place another order yet.")
    ).toBeInTheDocument();
    expect(screen.queryByText(/payment is verified/i)).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: /download invoice/i })).not.toBeInTheDocument();
    expect(screen.getByText("Invoice will be available after confirmation.")).toBeInTheDocument();
  });

  it("does not show verified success copy for needs_review payments", async () => {
    const page = await CheckoutSuccessPage({
      searchParams: Promise.resolve({ orderId: "9001", payment: "needs_review" }),
    });
    render(page);

    expect(screen.getByText("Your payment is under manual review")).toBeInTheDocument();
    expect(
      screen.getByText(
        "We received your payment update, but it needs manual verification. We'll contact you if action is needed."
      )
    ).toBeInTheDocument();
    expect(screen.queryByText(/payment is verified/i)).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: /download invoice/i })).not.toBeInTheDocument();
    expect(screen.getByText("Invoice will be available after confirmation.")).toBeInTheDocument();
  });

  it("shows invoice CTA only when invoice is marked available", async () => {
    const noInvoicePage = await CheckoutSuccessPage({
      searchParams: Promise.resolve({ orderId: "9001", payment: "paid" }),
    });
    render(noInvoicePage);

    expect(screen.queryByRole("link", { name: /download invoice/i })).not.toBeInTheDocument();
    expect(screen.getByText("Invoice will be available after confirmation.")).toBeInTheDocument();

    const withInvoicePage = await CheckoutSuccessPage({
      searchParams: Promise.resolve({ orderId: "9001", payment: "paid", invoice: "available" }),
    });
    render(withInvoicePage);

    expect(screen.getByRole("link", { name: /download invoice/i })).toBeInTheDocument();
  });
});
