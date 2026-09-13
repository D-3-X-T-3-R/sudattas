"use client";

import { useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { getCookieConsent, setCookieConsent } from "@/lib/cookie-consent";

// localStorage is an external system from React's point of view — useSyncExternalStore reads it
// safely (getServerSnapshot keeps SSR markup "undecided" so there's no hydration mismatch) without
// needing a setState-in-effect. No cross-tab subscription: a choice made in another tab won't
// live-hide this tab's banner, which is an acceptable trade-off for a one-time notice.
const noSubscription = () => () => {};

/** Storefront-only cookie-consent notice. Admin routes opt out, matching SiteHeader. */
export function CookieConsentBanner() {
  const pathname = usePathname() ?? "";
  const reduceMotion = !!useReducedMotion();
  const storedConsent = useSyncExternalStore(noSubscription, getCookieConsent, () => null);
  const [dismissed, setDismissed] = useState(false);
  const visible = storedConsent === null && !dismissed;

  function choose(choice: "all" | "necessary") {
    setCookieConsent(choice);
    setDismissed(true);
  }

  if (pathname.startsWith("/imtheboss")) return null;

  return (
    <AnimatePresence>
      {visible ? (
        <motion.div
          role="region"
          aria-label="Cookie notice"
          initial={reduceMotion ? undefined : { opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          exit={reduceMotion ? undefined : { opacity: 0, y: 24 }}
          transition={{ duration: 0.25, ease: "easeOut" }}
          className="fixed inset-x-3 bottom-[5.5rem] z-50 rounded-2xl border border-[var(--color-line)] bg-[var(--color-surface)]/98 p-4 shadow-[0_10px_30px_rgba(45,42,38,0.18)] backdrop-blur lg:inset-x-auto lg:bottom-6 lg:left-6 lg:max-w-sm"
        >
          <p className="text-sm leading-relaxed text-[var(--color-ink)]">
            We use cookies to keep your cart, sign-in, and checkout working — nothing more today.
            No analytics or advertising cookies are set. See our{" "}
            <Link href="/privacy-policy" className="underline underline-offset-2 hover:text-[var(--color-green)]">
              Privacy Policy
            </Link>{" "}
            for details.
          </p>
          <div className="mt-3 flex flex-wrap gap-2">
            <Button size="sm" onClick={() => choose("all")}>
              Accept all
            </Button>
            <Button size="sm" variant="outline" onClick={() => choose("necessary")}>
              Necessary only
            </Button>
          </div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
