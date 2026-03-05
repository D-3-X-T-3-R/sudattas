/**
 * Current JWT access token for API calls (set by Google OAuth / NextAuth).
 * Admin panel syncs session token here; GraphQL clients use it as Bearer when present.
 */

let accessToken: string | null = null;

export function setAccessToken(token: string | null): void {
  accessToken = token;
}

export function getAccessToken(): string | null {
  return accessToken;
}

export function clearAccessToken(): void {
  accessToken = null;
}
