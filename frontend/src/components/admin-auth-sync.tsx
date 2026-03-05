"use client";

import { useSession } from "next-auth/react";
import { useEffect } from "react";
import { setAccessToken, clearAccessToken } from "@/lib/authStore";

/**
 * When the admin panel is open and the user has a session, sync the session
 * token to authStore so gqlAdmin can send it as Bearer. Prefer id_token (JWT)
 * so the backend can validate it with Google JWKS.
 */
export function AdminAuthSync() {
  const { data: session, status } = useSession();

  useEffect(() => {
    if (status === "unauthenticated") {
      clearAccessToken();
      return;
    }
    const token = session?.idToken ?? session?.accessToken;
    setAccessToken(token ?? null);
  }, [session, status]);

  return null;
}
