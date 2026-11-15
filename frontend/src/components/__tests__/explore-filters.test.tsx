import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { ExploreSection } from "@/components/explore-section";
import { useExploreFilters, type StorefrontCatalogController } from "@/domains/storefront/hooks/use-storefront-catalog";
import { PRODUCTS_SEED } from "@/lib/seed-data";
vi.mock("@/components/scroll-reveal", () => ({ ScrollReveal: ({ children }: { children: ReactNode }) => <div>{children}</div> }));
vi.mock("@/components/product-card", () => ({ ProductCard: ({ product }: { product: { name: string } }) => <div>{product.name}</div> }));
const products = PRODUCTS_SEED.slice(0, 2).map((product, i) => ({ ...product, name: i ? "Medium product" : "Small product", variantStock: [{ variantId: String(i), sizeId: String(i), sizeName: i ? "M" : "S", quantity: 5 }] }));
const ignoreMood = async () => {};
function Harness() {
  const filters = useExploreFilters(products, null, ignoreMood);
  const catalog = { ...filters, products } as StorefrontCatalogController;
  return <ExploreSection catalog={catalog} wishlist={{}} onToggleWish={vi.fn()} onQuickView={vi.fn()} />;
}

it("keeps product results unchanged until APPLY and discards dismissed changes", async () => {
  const user = userEvent.setup();
  render(<Harness />);
  const trigger = screen.getByRole("button", { name: "Filters" });
  await user.click(trigger);
  let popup = screen.getByRole("dialog", { name: "Filters" });
  expect(within(popup).getByRole("button", { name: "RESET" })).toBeDisabled();
  expect(within(popup).queryByRole("button", { name: "Close" })).not.toBeInTheDocument();
  expect(within(popup).queryByText("Filter By")).not.toBeInTheDocument();
  await user.click(within(popup).getByRole("checkbox", { name: "S" }));
  expect(within(popup).getByRole("button", { name: "RESET" })).toBeEnabled();
  expect(screen.getByText("Medium product")).toBeInTheDocument();
  await user.keyboard("{Escape}");
  await user.click(trigger);
  popup = screen.getByRole("dialog", { name: "Filters" });
  expect(within(popup).getByRole("checkbox", { name: "S" })).not.toBeChecked();
  await user.click(within(popup).getByRole("checkbox", { name: "S" }));
  await user.click(within(popup).getByRole("button", { name: "APPLY" }));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  expect(screen.queryByText("Medium product")).not.toBeInTheDocument();
  expect(screen.getByText("Small product")).toBeInTheDocument();
  expect(trigger).toHaveFocus();
  await user.click(trigger);
  popup = screen.getByRole("dialog", { name: "Filters" });
  expect(within(popup).getByRole("checkbox", { name: "S" })).toBeChecked();
  await user.click(within(popup).getByRole("button", { name: "RESET" }));
  expect(screen.queryByText("Medium product")).not.toBeInTheDocument();
  await user.click(within(popup).getByRole("button", { name: "APPLY" }));
  expect(screen.getByText("Medium product")).toBeInTheDocument();
});
