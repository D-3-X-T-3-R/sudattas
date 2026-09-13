/**
 * Cookie-consent choice, persisted in localStorage.
 *
 * Today every cookie the site sets (cart/session, NextAuth sign-in, the admin JWT) is strictly
 * necessary for the site to function — there is no analytics/marketing pixel live yet. This
 * module exists anyway, ahead of that, so any future analytics/ad-pixel integration has a real
 * consent gate to check (`hasAnalyticsConsent()`) instead of being added first and bolting
 * consent on afterwards.
 */

const STORAGE_KEY = "sudattas_cookie_consent";

export type CookieConsentChoice = "all" | "necessary";

export function getCookieConsent(): CookieConsentChoice | null {
  if (typeof window === "undefined") return null;
  const raw = window.localStorage.getItem(STORAGE_KEY);
  return raw === "all" || raw === "necessary" ? raw : null;
}

export function setCookieConsent(choice: CookieConsentChoice): void {
  if (typeof window !== "undefined") {
    window.localStorage.setItem(STORAGE_KEY, choice);
  }
}

/** Gate for any future analytics/ad-pixel script: only load it once the visitor opted in. */
export function hasAnalyticsConsent(): boolean {
  return getCookieConsent() === "all";
}
