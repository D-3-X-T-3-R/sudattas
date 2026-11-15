"use client";

import type { Product } from "@/lib/schemas";
import type { ProductTransitionOrigin } from "@/components/product-card";

const OVERLAY_ID = "product-route-transition-image";
const BACKDROP_ID = "product-route-transition-backdrop";
const ORIGIN_KEY = "productRouteTransitionOrigin";
const EASE = "cubic-bezier(0.22, 1, 0.36, 1)";

type Router = { push: (href: string) => void; back: () => void };
type StoredOrigin = ProductTransitionOrigin & {
  productId: string;
  returnPath: string;
  scrollY: number;
  phase: "entering" | "on-pdp" | "returning";
};

function desktopViewport(): boolean {
  return window.innerWidth >= 768;
}

function clearTransitionState(): void {
  document.getElementById(OVERLAY_ID)?.remove();
  document.getElementById(BACKDROP_ID)?.remove();
  try { sessionStorage.removeItem(ORIGIN_KEY); } catch { /* ignore unavailable storage */ }
}

function reducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function readOrigin(productId: string): StoredOrigin | null {
  try {
    const value = JSON.parse(sessionStorage.getItem(ORIGIN_KEY) ?? "null") as StoredOrigin | null;
    return value?.productId === productId ? value : null;
  } catch {
    return null;
  }
}

function readStoredOrigin(): StoredOrigin | null {
  try {
    return JSON.parse(sessionStorage.getItem(ORIGIN_KEY) ?? "null") as StoredOrigin | null;
  } catch {
    return null;
  }
}

function writeOrigin(origin: StoredOrigin): void {
  try {
    sessionStorage.setItem(ORIGIN_KEY, JSON.stringify(origin));
  } catch { /* transitions still work without persisted state */ }
}

function estimatedPdpRect(): ProductTransitionOrigin["rect"] {
  const viewportWidth = window.innerWidth;
  const headerBottom = document.querySelector("header")?.getBoundingClientRect().bottom ?? 0;
  const mobile = viewportWidth < 768;
  const gutter = mobile ? 16 : 16;
  const pageWidth = viewportWidth - gutter * 2;
  const gridGap = viewportWidth >= 1024 ? 56 : 40;
  const galleryColumn = mobile ? pageWidth : (pageWidth - gridGap) * 0.6;
  const thumbnailRail = mobile ? 0 : 104;
  const width = Math.max(1, galleryColumn - thumbnailRail);
  return {
    top: headerBottom + (mobile ? 76 : 84),
    left: gutter + thumbnailRail,
    width,
    height: width * 1.5,
  };
}

function setRect(element: HTMLElement, rect: ProductTransitionOrigin["rect"]): void {
  element.style.top = `${rect.top}px`;
  element.style.left = `${rect.left}px`;
  element.style.width = `${rect.width}px`;
  element.style.height = `${rect.height}px`;
}

function makeOverlay(imageSrc: string, rect: ProductTransitionOrigin["rect"]): HTMLImageElement {
  document.getElementById(OVERLAY_ID)?.remove();
  const image = document.createElement("img");
  image.id = OVERLAY_ID;
  image.src = imageSrc;
  image.alt = "";
  image.style.position = "fixed";
  // Keep the route animation beneath the persistent header and mobile navigation (z-40).
  image.style.zIndex = "35";
  image.style.pointerEvents = "none";
  image.style.objectFit = "cover";
  image.style.borderRadius = "0.375rem";
  image.style.boxShadow = "0 22px 48px rgba(15, 61, 46, 0.18)";
  setRect(image, rect);
  document.body.appendChild(image);
  return image;
}

function makeBackdrop(): HTMLDivElement {
  document.getElementById(BACKDROP_ID)?.remove();
  const backdrop = document.createElement("div");
  backdrop.id = BACKDROP_ID;
  backdrop.style.position = "fixed";
  backdrop.style.inset = "0";
  backdrop.style.zIndex = "30";
  backdrop.style.pointerEvents = "none";
  backdrop.style.background = "#F6F3EA";
  backdrop.style.opacity = "0";
  document.body.appendChild(backdrop);
  return backdrop;
}

async function animate(element: Element, keyframes: Keyframe[], duration: number): Promise<void> {
  if (!("animate" in element) || reducedMotion()) return;
  await element.animate(keyframes, { duration, easing: EASE, fill: "forwards" }).finished.catch(() => undefined);
}

export function navigateToProductWithTransition(
  router: Router,
  product: Product,
  origin: ProductTransitionOrigin
): void {
  if (desktopViewport()) {
    clearTransitionState();
    router.push(`/product/${product.id}`);
    return;
  }

  try {
    writeOrigin({
      ...origin,
      productId: product.id,
      returnPath: `${window.location.pathname}${window.location.search}${window.location.hash}`,
      scrollY: window.scrollY,
      phase: "entering",
    });
  } catch { /* navigation still works without session storage */ }

  if (reducedMotion()) {
    router.push(`/product/${product.id}`);
    return;
  }

  const image = makeOverlay(origin.imageSrc, origin.rect);
  const backdrop = makeBackdrop();
  image.style.opacity = "1";
  const estimate = estimatedPdpRect();
  void animate(image, [origin.rect, estimate].map((rect) => ({
    top: `${rect.top}px`,
    left: `${rect.left}px`,
    width: `${rect.width}px`,
    height: `${rect.height}px`,
  })), 720);
  // Finish covering the listing before routing so the intermediate loading UI never shows.
  void animate(backdrop, [{ opacity: 0 }, { opacity: 1 }], 220).then(() => {
    router.push(`/product/${product.id}`);
  });
}

export async function finishProductPageTransition(productId: string): Promise<void> {
  if (desktopViewport()) {
    clearTransitionState();
    return;
  }
  const image = document.getElementById(OVERLAY_ID) as HTMLImageElement | null;
  const gallery = document.getElementById("pdp-main-image");
  const origin = readOrigin(productId);
  if (!origin) return;
  writeOrigin({ ...origin, phase: "on-pdp" });
  if (!image || !gallery) return;
  const rect = gallery.getBoundingClientRect();
  const target = { top: rect.top, left: rect.left, width: rect.width, height: rect.height };
  const currentRect = image.getBoundingClientRect();
  image.getAnimations?.().forEach((animation) => animation.cancel());
  setRect(image, {
    top: currentRect.top,
    left: currentRect.left,
    width: currentRect.width,
    height: currentRect.height,
  });
  const backdrop = document.getElementById(BACKDROP_ID);
  await Promise.all([
    animate(image, [
      { top: `${currentRect.top}px`, left: `${currentRect.left}px`, width: `${currentRect.width}px`, height: `${currentRect.height}px`, opacity: 1 },
      { top: `${target.top}px`, left: `${target.left}px`, width: `${target.width}px`, height: `${target.height}px`, opacity: 1, offset: 0.84 },
      { top: `${target.top}px`, left: `${target.left}px`, width: `${target.width}px`, height: `${target.height}px`, opacity: 0 },
    ], 620),
    ...(backdrop ? [animate(backdrop, [{ opacity: 1 }, { opacity: 0 }], 580)] : []),
  ]);
  image.remove();
  backdrop?.remove();
}

export async function leaveProductPageWithTransition(product: Product, router: Router): Promise<void> {
  if (desktopViewport()) {
    clearTransitionState();
    router.back();
    return;
  }
  const origin = readOrigin(product.id);
  const gallery = document.getElementById("pdp-main-image");
  if (!origin || !gallery || reducedMotion()) {
    if (origin) {
      writeOrigin({ ...origin, phase: "returning" });
      router.back();
    }
    else router.push("/");
    return;
  }
  const rect = gallery.getBoundingClientRect();
  const current = { top: rect.top, left: rect.left, width: rect.width, height: rect.height };
  makeOverlay(origin.imageSrc || product.image, current);
  const backdrop = makeBackdrop();
  writeOrigin({ ...origin, phase: "returning" });
  // Cover the PDP first, then let the source page render before shrinking the image back.
  await animate(backdrop, [{ opacity: 0 }, { opacity: 1 }], 320);
  router.back();
}

export function finishProductReturnTransition(): void {
  if (desktopViewport()) {
    clearTransitionState();
    return;
  }
  const origin = readStoredOrigin();
  if (!origin || origin.phase !== "returning" || window.location.pathname !== origin.returnPath.split(/[?#]/)[0]) return;
  const image = document.getElementById(OVERLAY_ID);
  const backdrop = document.getElementById(BACKDROP_ID);
  // The product grid's ScrollReveal starts 24px below its settled position.
  // Reusing the click-time coordinates avoids targeting that temporary transform.
  const restoreSourcePosition = () => window.scrollTo({ top: origin.scrollY, behavior: "auto" });
  restoreSourcePosition();
  window.requestAnimationFrame(() => {
    restoreSourcePosition();
    window.requestAnimationFrame(() => {
      restoreSourcePosition();
      void (async () => {
        const backdropFade = backdrop
          ? animate(backdrop, [{ opacity: 1 }, { opacity: 0 }], 420).then(() => backdrop.remove())
          : Promise.resolve();
        if (image) {
          const current = image.getBoundingClientRect();
          await animate(image, [
            { top: `${current.top}px`, left: `${current.left}px`, width: `${current.width}px`, height: `${current.height}px` },
            { top: `${origin.rect.top}px`, left: `${origin.rect.left}px`, width: `${origin.rect.width}px`, height: `${origin.rect.height}px` },
          ], window.innerWidth >= 768 ? 1800 : 1560);
          image.remove();
        }
        await backdropFade;
        try { sessionStorage.removeItem(ORIGIN_KEY); } catch { /* ignore unavailable storage */ }
      })();
    });
  });
}
