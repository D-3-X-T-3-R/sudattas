import { fireEvent, render, screen } from "@testing-library/react";
import { SiteHeader } from "@/components/site-header";
vi.mock("next/navigation", () => ({ usePathname: () => "/" }));
vi.mock("next-auth/react", () => ({ useSession: () => ({ status: "unauthenticated", data: null }) }));
vi.mock("framer-motion", () => ({ useReducedMotion: () => true }));
vi.mock("@/context/storefront-context", () => ({ useStorefront: () => ({ cartCount: 0, wishCount: 0 }) }));
vi.mock("@/context/storefront-login-context", () => ({ useStorefrontLogin: () => ({ openLogin: vi.fn() }) }));
vi.mock("@/hooks/use-lock-body-scroll", () => ({ useLockBodyScroll: vi.fn() }));
vi.mock("@/components/menu-drawer", () => ({ MenuDrawer: ({ onExitComplete }: { onExitComplete: () => void }) => <button onClick={onExitComplete}>Finish drawer exit</button> }));
vi.mock("@/components/header", () => ({
  Header: ({ searchOpen, setMenuOpen, menuOpen, menuPresent }: { searchOpen: boolean; setMenuOpen: (open: boolean) => void; menuOpen: boolean; menuPresent: boolean }) => <header data-testid="header" data-menu-present={menuPresent}>
    <button onClick={() => setMenuOpen(!menuOpen)}>{menuOpen ? "Close menu" : "Open menu"}</button>
    {searchOpen && <div data-site-search><input aria-label="Search products" /></div>}
  </header>,
}));

it("keeps search open inside its panel and dismisses it for another navigation action", () => {
  render(<SiteHeader />);
  fireEvent.click(screen.getByRole("button", { name: "Search" }));
  const input = screen.getByRole("textbox", { name: "Search products" });
  fireEvent.click(input);
  fireEvent.change(input, { target: { value: "silk" } });
  expect(input).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Sign in" }));
  expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Search" }));
  fireEvent.click(screen.getByRole("button", { name: "Open menu" }));
  expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
});

it("allows toggling search and dismissing with Escape", () => {
  render(<SiteHeader />);
  const search = screen.getByRole("button", { name: "Search" });
  fireEvent.click(search);
  expect(screen.getByRole("textbox")).toBeInTheDocument();
  fireEvent.click(search);
  expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  fireEvent.click(search);
  fireEvent.keyDown(document, { key: "Escape" });
  expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
});

it("keeps the toggle above the drawer until its exit animation completes", () => {
  render(<SiteHeader />);
  fireEvent.click(screen.getByRole("button", { name: "Open menu" }));
  expect(screen.getByTestId("header")).toHaveAttribute("data-menu-present", "true");
  fireEvent.click(screen.getByRole("button", { name: "Close menu" }));
  expect(screen.getByRole("button", { name: "Open menu" })).toBeInTheDocument();
  expect(screen.getByTestId("header")).toHaveAttribute("data-menu-present", "true");
  fireEvent.click(screen.getByRole("button", { name: "Finish drawer exit" }));
  expect(screen.getByTestId("header")).toHaveAttribute("data-menu-present", "false");
});
