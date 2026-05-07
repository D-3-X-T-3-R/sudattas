import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { ProductDetailView } from "@/components/product-detail-view";
import type { Product } from "@/lib/schemas";

vi.mock("next/image", () => ({
  default: (props: Record<string, unknown>) => {
    const { src = "", alt = "", ...rest } = props;
    delete rest.fill;
    delete rest.unoptimized;
    return React.createElement("img", { src: String(src), alt: String(alt), ...rest });
  },
}));

const product: Product = {
  id: "p-1",
  name: "Test Saree",
  collection: "Saree",
  price: 1299,
  pricePaise: 129900,
  rating: 4.2,
  reviews: 12,
  fabric: "Cotton",
  occasion: "Festive",
  description: "A test product",
  image: "/test.jpg",
  imageAlt: "Test Saree",
  variantStock: [
    { sizeId: "s", sizeName: "S", quantity: 2 },
    { sizeId: "m", sizeName: "M", quantity: 0 },
  ],
};

describe("ProductDetailView", () => {
  it("lets user adjust quantity and add selected size to bag", async () => {
    const user = userEvent.setup();
    const onAddToCart = vi.fn();

    render(
      <ProductDetailView
        product={product}
        sizes={[
          { sizeId: "s", sizeName: "S" },
          { sizeId: "m", sizeName: "M" },
        ]}
        wished={false}
        onToggleWish={vi.fn()}
        onAddToCart={onAddToCart}
      />
    );

    expect(screen.getByRole("button", { name: "M" })).toBeDisabled();
    expect(screen.getByRole("link", { name: /size & fit guide/i })).toHaveAttribute("href", "/size-fit-guide");

    await user.click(screen.getByRole("button", { name: "Increase quantity" }));
    await user.click(screen.getByRole("button", { name: "ADD TO BAG" }));

    expect(onAddToCart).toHaveBeenCalledWith(product, 2, "S");
  });

  it("shows fit guidance copy when standard size variants are unavailable", () => {
    render(
      <ProductDetailView
        product={{
          ...product,
          variantStock: [{ sizeId: "free", sizeName: "Free Size", quantity: 4 }],
        }}
        wished={false}
        onToggleWish={vi.fn()}
        onAddToCart={vi.fn()}
      />
    );

    expect(screen.getByText(/does not use standard size variants/i)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /view size & fit guide/i })).toHaveAttribute("href", "/size-fit-guide");
  });
});
