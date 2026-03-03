"use client";

import { useEffect, useRef } from "react";
import { setAccessToken, clearAccessToken } from "@/lib/authStore";

/**
 * Syncs Auth0 access token to authStore so GraphQL client can send Bearer token.
 * Only render when Auth0 is configured (e.g. inside Auth0Provider).
 * For Next.js we don't install @auth0/auth0-react by default; this component
 * is a no-op placeholder. Wire it to @auth0/nextjs-auth0 or your auth solution
 * and call setAccessToken(token) when the user is authenticated.
 */
export function AuthSync() {
  const synced = useRef(false);

  useEffect(() => {
    // Placeholder: no Auth0 in this app by default. When you add Auth0:
    // - Use getAccessTokenSilently() and call setAccessToken(token)
    // - On logout or unauthenticated, call clearAccessToken()
    if (!synced.current) {
      clearAccessToken();
    }
    return () => {
      synced.current = false;
    };
  }, []);

  return null;
}
