import { beforeEach, describe, expect, it } from "vitest";
import { getCookieConsent, hasAnalyticsConsent, setCookieConsent } from "@/lib/cookie-consent";

describe("cookie-consent helpers", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("returns null when no choice has been made yet", () => {
    expect(getCookieConsent()).toBeNull();
    expect(hasAnalyticsConsent()).toBe(false);
  });

  it("persists and round-trips an 'all' choice", () => {
    setCookieConsent("all");
    expect(getCookieConsent()).toBe("all");
    expect(hasAnalyticsConsent()).toBe(true);
  });

  it("persists and round-trips a 'necessary' choice", () => {
    setCookieConsent("necessary");
    expect(getCookieConsent()).toBe("necessary");
    expect(hasAnalyticsConsent()).toBe(false);
  });

  it("ignores garbage previously stored under the key", () => {
    localStorage.setItem("sudattas_cookie_consent", "garbage");
    expect(getCookieConsent()).toBeNull();
  });
});
