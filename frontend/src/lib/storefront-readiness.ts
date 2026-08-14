export const STOREFRONT_COMING_SOON_PATH = "/storefront-coming-soon";
export const STOREFRONT_GATE_HEADER = "x-sudattas-storefront-gated";
export const STOREFRONT_GATE_REASON_HEADER = "x-sudattas-storefront-gate-reason";
export const STOREFRONT_GATE_REASON_PARAM = "reason";
export const STOREFRONT_BACKEND_HEALTH_TIMEOUT_MS = 1500;
export const STOREFRONT_BACKEND_HEALTHY_CACHE_MS = 5_000;
export const STOREFRONT_BACKEND_UNHEALTHY_CACHE_MS = 3_000;

const PUBLIC_FILE_PATTERN = /\.(?:avif|bmp|css|csv|gif|ico|jpg|jpeg|js|json|map|mp4|png|svg|txt|webm|webp|woff|woff2|xml)$/i;
const DEFAULT_GRAPHQL_URL = "http://localhost:8080/v2";

export type StorefrontGateReason = "not-ready" | "service-unavailable";

type BackendAvailabilityCache = {
  available: boolean;
  expiresAt: number;
  url: string;
};

let backendAvailabilityCache: BackendAvailabilityCache | null = null;
let backendAvailabilityInFlight: Promise<boolean> | null = null;

export function isStorefrontReady(value = process.env.IS_STOREFRONT_READY): boolean {
  return value === "1";
}

function configuredGraphqlUrl(): string {
  return (
    process.env.GRAPHQL_URL ??
    process.env.NEXT_PUBLIC_GRAPHQL_URL ??
    DEFAULT_GRAPHQL_URL
  );
}

export function storefrontBackendHealthUrl(
  graphqlUrl = configuredGraphqlUrl()
): string | null {
  try {
    const url = new URL(graphqlUrl);
    const basePath = url.pathname.replace(/\/v2\/?$/, "").replace(/\/$/, "");
    url.pathname = `${basePath}/ready`;
    url.search = "";
    url.hash = "";
    return url.toString();
  } catch {
    return null;
  }
}

export function resetStorefrontBackendAvailabilityCache(): void {
  backendAvailabilityCache = null;
  backendAvailabilityInFlight = null;
}

async function fetchBackendAvailability(
  healthUrl: string,
  timeoutMs: number
): Promise<boolean> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetch(healthUrl, {
      method: "GET",
      cache: "no-store",
      headers: {
        accept: "text/plain",
      },
      signal: controller.signal,
    });

    return response.ok;
  } catch {
    return false;
  } finally {
    clearTimeout(timeout);
  }
}

export async function isStorefrontBackendAvailable({
  now = Date.now(),
  timeoutMs = STOREFRONT_BACKEND_HEALTH_TIMEOUT_MS,
}: {
  now?: number;
  timeoutMs?: number;
} = {}): Promise<boolean> {
  // Browser-only E2E suites (e.g. critical-browser-e2e) intentionally run with no real
  // backend, mocking every API call at the browser level via page.route(); this middleware
  // check runs server-side and can't be reached by those mocks. Explicit, narrowly-scoped
  // opt-out for exactly that scenario. Gated to non-production so a stray copy of this env
  // var into a prod config can never mask a real backend outage from customers.
  if (
    process.env.SKIP_STOREFRONT_BACKEND_HEALTH_CHECK === "1" &&
    process.env.NODE_ENV !== "production"
  ) {
    return true;
  }

  const healthUrl = storefrontBackendHealthUrl();

  if (!healthUrl) {
    return false;
  }

  if (
    backendAvailabilityCache &&
    backendAvailabilityCache.url === healthUrl &&
    backendAvailabilityCache.expiresAt > now
  ) {
    return backendAvailabilityCache.available;
  }

  if (backendAvailabilityInFlight) {
    return backendAvailabilityInFlight;
  }

  backendAvailabilityInFlight = fetchBackendAvailability(healthUrl, timeoutMs).then(
    (available) => {
      backendAvailabilityCache = {
        available,
        expiresAt:
          Date.now() +
          (available
            ? STOREFRONT_BACKEND_HEALTHY_CACHE_MS
            : STOREFRONT_BACKEND_UNHEALTHY_CACHE_MS),
        url: healthUrl,
      };
      backendAvailabilityInFlight = null;
      return available;
    },
    () => {
      backendAvailabilityCache = {
        available: false,
        expiresAt: Date.now() + STOREFRONT_BACKEND_UNHEALTHY_CACHE_MS,
        url: healthUrl,
      };
      backendAvailabilityInFlight = null;
      return false;
    }
  );

  return backendAvailabilityInFlight;
}

export function isAdminPath(pathname: string): boolean {
  return pathname === "/imtheboss" || pathname.startsWith("/imtheboss/");
}

export function isNextInternalPath(pathname: string): boolean {
  return pathname === "/favicon.ico" || pathname.startsWith("/_next/");
}

export function isStaticAssetPath(pathname: string): boolean {
  return PUBLIC_FILE_PATTERN.test(pathname);
}

export function isApiPath(pathname: string): boolean {
  return pathname === "/api" || pathname.startsWith("/api/");
}

export function isComingSoonPath(pathname: string): boolean {
  return pathname === STOREFRONT_COMING_SOON_PATH;
}

export function isPublicStorefrontPath(pathname: string): boolean {
  if (isComingSoonPath(pathname)) return false;
  if (isAdminPath(pathname)) return false;
  if (isApiPath(pathname)) return false;
  if (isNextInternalPath(pathname)) return false;
  if (isStaticAssetPath(pathname)) return false;
  return true;
}

export function normalizeStorefrontGateReason(
  value: unknown
): StorefrontGateReason {
  return value === "service-unavailable" ? "service-unavailable" : "not-ready";
}

export function storefrontGateReasonForPath(
  pathname: string,
  ready = isStorefrontReady(),
  backendAvailable = true
): StorefrontGateReason | null {
  if (!isPublicStorefrontPath(pathname)) return null;
  if (!ready) return "not-ready";
  if (!backendAvailable) return "service-unavailable";
  return null;
}

export function shouldGateStorefrontPath(
  pathname: string,
  ready = isStorefrontReady(),
  backendAvailable = true
): boolean {
  return storefrontGateReasonForPath(pathname, ready, backendAvailable) !== null;
}
