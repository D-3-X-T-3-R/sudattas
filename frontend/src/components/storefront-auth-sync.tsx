"use client";

import { useSession } from "next-auth/react";
import { useEffect } from "react";
import { setAccessToken, clearAccessToken } from "@/lib/authStore";
import { ensureGuestSession } from "@/lib/session";

/**
 * Syncs NextAuth tokens into authStore so storefront `gql()` sends Bearer (Google id_token)
 * for cart, wishlist, checkout, and profile — same pattern as AdminAuthSync.
 */
export function StorefrontAuthSync() {
  const { data: session, status } = useSession();

  useEffect(() => {
    const notify = () => {
      if (typeof window !== "undefined") {
        window.dispatchEvent(new Event("sudattas-auth-changed"));
      }
    };

    if (status === "unauthenticated") {
      clearAccessToken();
      void ensureGuestSession().finally(notify);
      return;
    }
    const token = session?.idToken ?? session?.accessToken;
    setAccessToken(token ?? null);
    notify();
  }, [session, status]);

  return null;
}
