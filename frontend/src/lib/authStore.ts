/**
 * Current JWT access token for API calls (set by Auth0 sync).
 * GraphQL client reads this when present so requests use Bearer token.
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
