import CheckoutAddressPage from "@/app/checkout/address/page";

const redirectMock = vi.fn();

vi.mock("next/navigation", () => ({
  redirect: (...args: unknown[]) => redirectMock(...args),
}));

describe("CheckoutAddressPage", () => {
  beforeEach(() => {
    redirectMock.mockReset();
  });

  it("redirects to /bag immediately", () => {
    CheckoutAddressPage();
    expect(redirectMock).toHaveBeenCalledWith("/bag");
  });
});

