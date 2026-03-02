/**
 * Syncs Auth0 access token to authStore so api.js can send Bearer token.
 * Only render when Auth0 is configured (inside Auth0Provider).
 */
import { useEffect, useRef } from "react";
import { useAuth0 } from "@auth0/auth0-react";
import { clearAccessToken, setAccessToken } from "./authStore";

export default function AuthSync() {
  const { isAuthenticated, getAccessTokenSilently } = useAuth0();
  const synced = useRef(false);

  useEffect(() => {
    if (!isAuthenticated) {
      clearAccessToken();
      synced.current = false;
      return;
    }
    let cancelled = false;
    getAccessTokenSilently({
      authorizationParams: {
        audience: process.env.REACT_APP_AUTH0_AUDIENCE || undefined,
      },
    })
      .then((token) => {
        if (!cancelled) {
          setAccessToken(token);
          synced.current = true;
        }
      })
      .catch(() => {
        if (!cancelled) clearAccessToken();
      });
    return () => {
      cancelled = true;
    };
  }, [isAuthenticated, getAccessTokenSilently]);

  return null;
}
