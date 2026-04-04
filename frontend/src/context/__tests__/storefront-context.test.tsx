import { render } from "@testing-library/react";
import { StorefrontProvider } from "@/context/storefront-context";

const useSessionMock = vi.fn();
const useStorefrontWishlistMock = vi.fn();
const useStorefrontCartMock = vi.fn();
const reloadCartFromBackendMock = vi.fn();

vi.mock("next-auth/react", () => ({
  useSession: () => useSessionMock(),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({ showToast: vi.fn() }),
}));

vi.mock("@/components/ui/live-announcer", () => ({
  useLiveAnnouncer: () => ({ announce: vi.fn() }),
}));

vi.mock("@/domains/storefront/hooks/use-storefront-wishlist", () => ({
  useStorefrontWishlist: () => useStorefrontWishlistMock(),
}));

vi.mock("@/domains/storefront/hooks/use-storefront-cart", () => ({
  useStorefrontCart: () => useStorefrontCartMock(),
}));

describe("StorefrontProvider", () => {
  beforeEach(() => {
    reloadCartFromBackendMock.mockReset();
    useSessionMock.mockReturnValue({ status: "authenticated", data: null });
    useStorefrontWishlistMock.mockReturnValue({
      wishlist: {},
      toggleWish: vi.fn(),
    });
    useStorefrontCartMock.mockReturnValue({
      cart: {},
      cartLoading: false,
      cartLines: [],
      addToCart: vi.fn(),
      decCart: vi.fn(),
      incCart: vi.fn(),
      removeCart: vi.fn(),
      reloadCartFromBackend: reloadCartFromBackendMock,
    });
  });

  it("reloads cart when auth-changed event is emitted (cart merge path)", () => {
    render(
      <StorefrontProvider>
        <div>child</div>
      </StorefrontProvider>
    );

    window.dispatchEvent(new Event("sudattas-auth-changed"));
    expect(reloadCartFromBackendMock).toHaveBeenCalledTimes(1);
  });
});
