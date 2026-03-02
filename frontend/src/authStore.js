/**
 * Current JWT access token for API calls (set by Auth0 sync).
 * api.js reads this when present so GraphQL requests use Bearer token.
 */
let accessToken = null;

export function setAccessToken(token) {
  accessToken = token;
}

export function getAccessToken() {
  return accessToken;
}

export function clearAccessToken() {
  accessToken = null;
}
