/**
 * Current JWT access token for API calls (set by Google OAuth / NextAuth).
 * Admin panel syncs session token here; GraphQL clients use it as Bearer when present.
 */

let accessToken: string | null = null;
let customerUserId: string | null = null;

export function setAccessToken(token: string | null): void {
  accessToken = token;
}

export function getAccessToken(): string | null {
  return accessToken;
}

export function setCustomerUserId(userId: string | null): void {
  customerUserId = userId;
}

export function getCustomerUserId(): string | null {
  return customerUserId;
}

export function clearAccessToken(): void {
  accessToken = null;
  customerUserId = null;
}
