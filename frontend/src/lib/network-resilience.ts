const DEFAULT_BASE_BACKOFF_MS = 400;
const DEFAULT_JITTER_MAX_MS = 250;

export type RetryPolicy = {
  max429Retries?: number;
  maxNetworkRetries?: number;
  baseBackoffMs?: number;
  jitterMaxMs?: number;
};

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function parseRetryAfterMs(headerValue: string | null): number | null {
  if (!headerValue) return null;
  const asSeconds = Number.parseInt(headerValue, 10);
  if (Number.isFinite(asSeconds) && asSeconds > 0) return asSeconds * 1000;

  const asDate = Date.parse(headerValue);
  if (Number.isFinite(asDate)) {
    const delta = asDate - Date.now();
    if (delta > 0) return delta;
  }
  return null;
}

function jitteredBackoffMs(attempt: number, policy: RetryPolicy): number {
  const base = policy.baseBackoffMs ?? DEFAULT_BASE_BACKOFF_MS;
  const jitterMax = policy.jitterMaxMs ?? DEFAULT_JITTER_MAX_MS;
  const exp = base * 2 ** Math.max(0, attempt - 1);
  const jitter = Math.floor(Math.random() * jitterMax);
  return exp + jitter;
}

function shouldRetryNetworkError(error: unknown): boolean {
  if (!(error instanceof Error)) return false;
  const m = error.message.toLowerCase();
  return (
    m.includes("fetch failed") ||
    m.includes("networkerror") ||
    m.includes("network error") ||
    m.includes("econnreset") ||
    m.includes("etimedout") ||
    m.includes("socket") ||
    m.includes("timeout")
  );
}

export async function fetchWithResilience(
  input: RequestInfo | URL,
  init: RequestInit,
  policy: RetryPolicy = {}
): Promise<Response> {
  const max429Retries = Math.max(0, policy.max429Retries ?? 1);
  const maxNetworkRetries = Math.max(0, policy.maxNetworkRetries ?? 1);
  let attempt429 = 0;
  let attemptNetwork = 0;

  while (true) {
    try {
      const response = await fetch(input, init);
      if (response.status === 429 && attempt429 < max429Retries) {
        attempt429 += 1;
        const waitMs =
          parseRetryAfterMs(response.headers.get("retry-after")) ??
          jitteredBackoffMs(attempt429, policy);
        await sleep(waitMs);
        continue;
      }
      return response;
    } catch (error) {
      if (!shouldRetryNetworkError(error) || attemptNetwork >= maxNetworkRetries) {
        throw error;
      }
      attemptNetwork += 1;
      await sleep(jitteredBackoffMs(attemptNetwork, policy));
    }
  }
}

export function normalizeNetworkError(error: unknown, fallback = "Network request failed"): string {
  if (error instanceof Error) {
    const message = error.message?.trim();
    if (message) return message;
  }
  return fallback;
}
