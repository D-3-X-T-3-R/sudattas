import { useEffect } from "react";
import { clearPendingHomeSection, goTo, peekPendingHomeSection } from "@/hooks/use-scroll-to";

type UseStorefrontNavigationEffectsProps = {
  pathname: string;
  reduceMotion: boolean;
  loadingProducts: boolean;
};

export function useStorefrontNavigationEffects({
  pathname,
  reduceMotion,
  loadingProducts,
}: UseStorefrontNavigationEffectsProps) {
  useEffect(() => {
    if (typeof window === "undefined") return;
    const previous = window.history.scrollRestoration;
    window.history.scrollRestoration = "manual";
    return () => {
      window.history.scrollRestoration = previous;
    };
  }, []);

  useEffect(() => {
    if (pathname !== "/") return;

    const hashId = typeof window !== "undefined" ? window.location.hash.slice(1) : "";
    const { id: pendingId, fromOtherPage } = peekPendingHomeSection();
    const id = hashId || pendingId;
    if (!id) return;

    const scrollViaTop = Boolean(pendingId) && fromOtherPage && !hashId;
    let cancelled = false;
    let attempts = 0;

    const finish = () => {
      if (!pendingId) return;
      const waitForCatalog = loadingProducts && (id === "shop" || id === "explore");
      if (waitForCatalog) return;
      clearPendingHomeSection();
    };

    const tryScroll = () => {
      if (cancelled) return;
      if (document.getElementById(id)) {
        if (scrollViaTop) {
          window.scrollTo({ top: 0, behavior: "auto" });
          requestAnimationFrame(() => {
            requestAnimationFrame(() => {
              if (!cancelled) {
                goTo(id, false);
                finish();
              }
            });
          });
        } else {
          goTo(id, reduceMotion);
          finish();
        }
        return;
      }
      attempts += 1;
      if (attempts < 120) window.setTimeout(tryScroll, 50);
    };

    tryScroll();
    return () => {
      cancelled = true;
    };
  }, [pathname, reduceMotion, loadingProducts]);

  useEffect(() => {
    const onHashChange = () => {
      if (window.location.pathname !== "/") return;
      const id = window.location.hash.slice(1);
      if (!id) return;
      goTo(id, reduceMotion);
    };
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, [reduceMotion]);
}
