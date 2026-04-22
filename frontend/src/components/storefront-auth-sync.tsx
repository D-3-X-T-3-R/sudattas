"use client";

import { useSession } from "next-auth/react";
import { useEffect } from "react";
import { setAccessToken, clearAccessToken, setCustomerUserId } from "@/lib/authStore";
import { ensureGuestSession, getGuestSessionId } from "@/lib/session";

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
    const resolvedCustomerUserId = (session as { customerUserId?: string } | null)?.customerUserId ?? null;
    setCustomerUserId(resolvedCustomerUserId);
    const guestSessionId = getGuestSessionId();
    if (resolvedCustomerUserId && guestSessionId) {
      void fetch("/api/account/cart/merge", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Idempotency-Key": `cart-merge-${resolvedCustomerUserId}-${guestSessionId}`,
          "X-Guest-Session-Id": guestSessionId,
        },
        body: JSON.stringify({ guestSessionId }),
      }).finally(notify);
      return;
    }
    notify();
  }, [session, status]);

  return null;
}
