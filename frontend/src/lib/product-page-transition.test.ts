import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  finishProductPageTransition,
  finishProductReturnTransition,
  leaveProductPageWithTransition,
  navigateToProductWithTransition,
} from "@/lib/product-page-transition";
import type { Product } from "@/lib/schemas";

const product = {
  id: "transition-product",
  name: "Transition Saree",
  collection: "Sarees",
  price: 1000,
  rating: 0,
  reviews: 0,
  fabric: "Silk",
  occasion: "Festive",
  description: "Description",
  image: "/transition.jpg",
  imageAlt: "Transition Saree",
} satisfies Product;

beforeEach(() => {
  sessionStorage.clear();
  Object.defineProperty(window, "innerWidth", { configurable: true, value: 390 });
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: () => ({ matches: false }),
  });
  Object.defineProperty(window, "scrollY", { configurable: true, value: 640 });
  Object.defineProperty(window, "scrollTo", { configurable: true, value: vi.fn() });
  Object.defineProperty(window, "requestAnimationFrame", {
    configurable: true,
    value: (callback: FrameRequestCallback) => { callback(0); return 1; },
  });
  Object.defineProperty(Element.prototype, "animate", {
    configurable: true,
    value: vi.fn(() => ({ finished: Promise.resolve() })),
  });
});

afterEach(() => {
  document.getElementById("product-route-transition-image")?.remove();
  document.getElementById("product-route-transition-backdrop")?.remove();
  document.getElementById("pdp-main-image")?.remove();
  document.querySelectorAll("[data-product-transition-key]").forEach((element) => element.remove());
});

it("animates the card image, navigates to the existing PDP, and hands off to its gallery", async () => {
  const router = { push: vi.fn(), back: vi.fn() };
  navigateToProductWithTransition(router, product, {
    imageSrc: product.image,
    transitionKey: "explore:transition-product",
    rect: { top: 100, left: 20, width: 160, height: 240 },
  });

  await vi.waitFor(() => expect(router.push).toHaveBeenCalledWith("/product/transition-product"));
  expect(document.getElementById("product-route-transition-image")).toBeInTheDocument();
  expect(document.getElementById("product-route-transition-image")).toHaveStyle({ zIndex: "35" });
  expect(document.getElementById("product-route-transition-backdrop")).toHaveStyle({ zIndex: "30" });

  const gallery = document.createElement("div");
  gallery.id = "pdp-main-image";
  gallery.getBoundingClientRect = () => ({ top: 80, left: 16, width: 320, height: 480, right: 336, bottom: 560, x: 16, y: 80, toJSON: () => ({}) });
  document.body.appendChild(gallery);
  await finishProductPageTransition(product.id);

  expect(document.getElementById("product-route-transition-image")).not.toBeInTheDocument();
  expect(document.getElementById("product-route-transition-backdrop")).not.toBeInTheDocument();

  await leaveProductPageWithTransition(product, router);
  expect(router.back).toHaveBeenCalledOnce();
  expect(document.getElementById("product-route-transition-backdrop")).toBeInTheDocument();
  const sourceCard = document.createElement("a");
  sourceCard.dataset.productTransitionKey = "explore:transition-product";
  sourceCard.getBoundingClientRect = () => ({ top: 100, left: 700, width: 300, height: 450, right: 1000, bottom: 550, x: 700, y: 100, toJSON: () => ({}) });
  document.body.appendChild(sourceCard);
  finishProductReturnTransition();
  expect(window.scrollTo).toHaveBeenCalledWith({ top: 640, behavior: "auto" });
  expect(Element.prototype.animate).toHaveBeenCalledWith(
    expect.arrayContaining([expect.objectContaining({ left: "20px", width: "160px" })]),
    expect.any(Object)
  );
  await vi.waitFor(() => expect(document.getElementById("product-route-transition-image")).not.toBeInTheDocument());
});

it("uses normal navigation without transition overlays on desktop", async () => {
  Object.defineProperty(window, "innerWidth", { configurable: true, value: 1200 });
  const router = { push: vi.fn(), back: vi.fn() };

  navigateToProductWithTransition(router, product, {
    imageSrc: product.image,
    transitionKey: "explore:transition-product",
    rect: { top: 100, left: 700, width: 300, height: 450 },
  });

  expect(router.push).toHaveBeenCalledWith("/product/transition-product");
  expect(document.getElementById("product-route-transition-image")).not.toBeInTheDocument();
  expect(document.getElementById("product-route-transition-backdrop")).not.toBeInTheDocument();

  await leaveProductPageWithTransition(product, router);
  expect(router.back).toHaveBeenCalledOnce();
  expect(document.getElementById("product-route-transition-image")).not.toBeInTheDocument();
});
