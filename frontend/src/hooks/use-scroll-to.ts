"use client";

const PENDING_HOME_SECTION_KEY = "pendingHomeSection";
const PENDING_HOME_FROM_OTHER_PAGE_KEY = "pendingHomeFromOtherPage";
const PENDING_HOME_COLLECTION_KEY = "pendingHomeCollection";
export const PENDING_HOME_COLLECTION_EVENT = "pending-home-collection";

/** Used before `router.push('/')` from bag/wishlist/etc. */
export function setPendingHomeSection(
  id: string,
  options?: { fromOtherPage?: boolean }
): void {
  try {
    sessionStorage.setItem(PENDING_HOME_SECTION_KEY, id);
    if (options?.fromOtherPage) {
      sessionStorage.setItem(PENDING_HOME_FROM_OTHER_PAGE_KEY, "1");
    }
  } catch {
    /* private mode */
  }
}

/**
 * Read pending nav without clearing. Use this in effects that may run twice (React Strict Mode);
 * call `clearPendingHomeSection()` after a successful scroll.
 */
export function peekPendingHomeSection(): {
  id: string | null;
  fromOtherPage: boolean;
} {
  try {
    const id = sessionStorage.getItem(PENDING_HOME_SECTION_KEY);
    const fromOther = sessionStorage.getItem(PENDING_HOME_FROM_OTHER_PAGE_KEY);
    return { id: id ?? null, fromOtherPage: fromOther === "1" };
  } catch {
    return { id: null, fromOtherPage: false };
  }
}

export function clearPendingHomeSection(): void {
  try {
    sessionStorage.removeItem(PENDING_HOME_SECTION_KEY);
    sessionStorage.removeItem(PENDING_HOME_FROM_OTHER_PAGE_KEY);
  } catch {
    /* private mode */
  }
}

export function setPendingHomeCollection(collection: string): void {
  try {
    sessionStorage.setItem(PENDING_HOME_COLLECTION_KEY, collection);
  } catch {
    /* private mode */
  }
  try {
    window.dispatchEvent(new CustomEvent(PENDING_HOME_COLLECTION_EVENT, { detail: collection }));
  } catch {
    /* browser-only */
  }
}

export function consumePendingHomeCollection(): string | null {
  try {
    const collection = sessionStorage.getItem(PENDING_HOME_COLLECTION_KEY);
    sessionStorage.removeItem(PENDING_HOME_COLLECTION_KEY);
    return collection;
  } catch {
    return null;
  }
}

function stickyHeaderOffsetPx(): number {
  const header = document.querySelector("header");
  if (header instanceof HTMLElement) {
    return Math.ceil(header.getBoundingClientRect().height);
  }
  return 88;
}

function documentScrollY(): number {
  return (
    window.scrollY ||
    window.pageYOffset ||
    document.documentElement.scrollTop ||
    document.body.scrollTop ||
    0
  );
}

/**
 * Scroll so the section sits under the sticky header (not vertically centered).
 * Tall `<section>`s + native `/#hash` scrolling can fight this; callers may schedule retries.
 */
export function goTo(id: string, instant = false): void {
  if (id === "top") {
    window.scrollTo({ top: 0, behavior: instant ? "auto" : "smooth" });
    return;
  }
  const el = document.getElementById(id);
  if (!el) return;
  const offset = stickyHeaderOffsetPx();
  const top = el.getBoundingClientRect().top + documentScrollY() - offset;
  window.scrollTo({ top: Math.max(0, top), behavior: instant ? "auto" : "smooth" });
}
