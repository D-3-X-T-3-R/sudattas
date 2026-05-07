import { render, screen } from "@testing-library/react";
import React from "react";
import { QuickViewModal } from "@/components/quick-view-modal";
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
  id: "p-quick",
  name: "Quick View Saree",
  collection: "Saree",
  price: 2499,
  pricePaise: 249900,
  rating: 4.8,
  reviews: 24,
  fabric: "Silk",
  occasion: "Wedding",
  description: "Test quick view description",
  image: "/quick.jpg",
  imageAlt: "Quick View Saree",
  variantStock: [{ sizeId: "free", sizeName: "Free Size", quantity: 2 }],
};

describe("QuickViewModal", () => {
  it("wires size guidance CTA to size & fit guide", () => {
    render(
      <QuickViewModal
        product={product}
        open
        onClose={vi.fn()}
        wished={false}
        onToggleWish={vi.fn()}
        onAddToCart={vi.fn()}
      />
    );

    expect(screen.getByRole("link", { name: /view size & fit guide/i })).toHaveAttribute(
      "href",
      "/size-fit-guide"
    );
    expect(screen.getByText(/does not use standard size variants/i)).toBeInTheDocument();
  });
});
