import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { ProductCard } from "@/components/product-card";
import type { Product } from "@/lib/schemas";

vi.mock("next/image", () => ({
  default: (props: Record<string, unknown>) => {
    const { src = "", alt = "", ...rest } = props;
    delete rest.fill;
    delete rest.unoptimized;
    return React.createElement("img", { src: String(src), alt: String(alt), ...rest });
  },
}));

it("passes the clicked image geometry into the PDP route transition", async () => {
  Object.defineProperty(window, "innerWidth", { configurable: true, value: 390 });
  const user = userEvent.setup();
  const product = {
    id: "animated-product",
    name: "Animated Saree",
    collection: "Sarees",
    price: 1200,
    rating: 0,
    reviews: 0,
    fabric: "Silk",
    occasion: "Festive",
    description: "Product description",
    image: "/animated.jpg",
    imageAlt: "Animated Saree",
  } satisfies Product;
  const onQuickView = vi.fn();
  render(<ProductCard product={product} wished={false} onToggleWish={vi.fn()} onQuickView={onQuickView} />);

  await user.click(screen.getByRole("link", { name: "View Animated Saree" }));

  expect(onQuickView).toHaveBeenCalledWith(product, {
    imageSrc: "/animated.jpg",
    transitionKey: "products:animated-product",
    rect: { top: 0, left: 0, width: 0, height: 0 },
  });
});
