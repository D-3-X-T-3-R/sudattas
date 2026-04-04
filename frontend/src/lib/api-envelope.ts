import { classifyStatusError, trackClientTelemetry } from "@/lib/client-telemetry";

export type ApiEnvelope<T> = {
  ok: boolean;
  data: T | null;
  errorCode: string | null;
  message: string | null;
  fieldErrors: Record<string, string> | null;
  retryable: boolean;
};

let sessionRefreshInFlight: Promise<void> | null = null;
const GUEST_SESSION_STORAGE_KEY = "sudattas_guest_session";

function randomRequestId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `req-${Date.now()}-${Math.floor(Math.random() * 1_000_000)}`;
}

function toAbsoluteUrl(input: RequestInfo | URL): URL | null {
  if (input instanceof URL) return input;
  if (typeof input === "string") {
    try {
      if (typeof window !== "undefined") return new URL(input, window.location.origin);
      return new URL(input);
    } catch {
      return null;
    }
  }
  if (typeof Request !== "undefined" && input instanceof Request) {
    try {
      if (typeof window !== "undefined") return new URL(input.url, window.location.origin);
      return new URL(input.url);
    } catch {
      return null;
    }
  }
  return null;
}

function normalizedMethod(init?: RequestInit): string {
  return (init?.method ?? "GET").toUpperCase();
}

function inferClientAction(input: RequestInfo | URL, init?: RequestInit): string {
  const method = normalizedMethod(init);
  const url = toAbsoluteUrl(input);
  const path = url?.pathname ?? "unknown";
  return `${method} ${path}`;
}

function pathFromInput(input: RequestInfo | URL): string {
  const url = toAbsoluteUrl(input);
  return url?.pathname ?? "unknown";
}

function requestIdFromInit(init: RequestInit): string | null {
  const headers = new Headers(init.headers ?? undefined);
  return headers.get("X-Request-Id");
}

function shouldTrackSuccess(route: string): boolean {
  return (
    route.startsWith("/api/checkout/") ||
    route.startsWith("/api/admin/") ||
    route === "/api/products" ||
    route === "/api/storefront-filters"
  );
}

function guestSessionIdForHeader(): string | null {
  if (typeof window === "undefined") return null;
  try {
    const id = window.localStorage.getItem(GUEST_SESSION_STORAGE_KEY)?.trim() ?? "";
    return id || null;
  } catch {
    return null;
  }
}

function mergeHeadersWithClientMetadata(
  input: RequestInfo | URL,
  init?: RequestInit
): RequestInit {
  const headers = new Headers(init?.headers ?? undefined);
  if (!headers.has("X-Request-Id")) headers.set("X-Request-Id", randomRequestId());
  if (!headers.has("X-Client-Action")) {
    headers.set("X-Client-Action", inferClientAction(input, init));
  }
  const sid = guestSessionIdForHeader();
  if (sid && !headers.has("X-Guest-Session-Id")) {
    headers.set("X-Guest-Session-Id", sid);
  }
  return { ...(init ?? {}), headers };
}

async function tryRefreshBrowserSession(): Promise<void> {
  if (typeof window === "undefined") return;
  if (sessionRefreshInFlight) return sessionRefreshInFlight;
  sessionRefreshInFlight = (async () => {
    try {
      await fetch("/api/auth/session", {
        method: "GET",
        cache: "no-store",
        credentials: "same-origin",
      });
    } catch {
      // Swallow refresh errors; caller will surface original auth failure if retry still fails.
    }
  })().finally(() => {
    sessionRefreshInFlight = null;
  });
  return sessionRefreshInFlight;
}

export class ApiEnvelopeError extends Error {
  readonly status: number;
  readonly errorCode: string | null;
  readonly fieldErrors: Record<string, string> | null;
  readonly retryable: boolean;

  constructor(params: {
    message: string;
    status: number;
    errorCode: string | null;
    fieldErrors: Record<string, string> | null;
    retryable: boolean;
  }) {
    super(params.message);
    this.name = "ApiEnvelopeError";
    this.status = params.status;
    this.errorCode = params.errorCode;
    this.fieldErrors = params.fieldErrors;
    this.retryable = params.retryable;
  }
}

export async function fetchApiEnvelope<T>(
  input: RequestInfo | URL,
  init?: RequestInit,
  attemptedRefresh = false
): Promise<T> {
  const requestInit = mergeHeadersWithClientMetadata(input, init);
  const action = inferClientAction(input, requestInit);
  const route = pathFromInput(input);
  const requestId = requestIdFromInit(requestInit);
  const mode =
    route.startsWith("/api/admin") ? "admin" : route.startsWith("/api/account") || route.startsWith("/api/checkout") ? "account" : "public";
  const response = await fetch(input, requestInit);
  if (response.status === 401 && !attemptedRefresh) {
    await tryRefreshBrowserSession();
    return fetchApiEnvelope<T>(input, init, true);
  }
  const text = await response.text();

  let parsed: ApiEnvelope<T> | null = null;
  try {
    parsed = JSON.parse(text) as ApiEnvelope<T>;
  } catch {
    trackClientTelemetry({
      route,
      userMode: mode,
      action,
      errorClass: classifyStatusError(response.status, response.status >= 500, null),
      errorCode: null,
      message: text || `HTTP ${response.status}`,
      status: response.status,
      requestId,
    });
    throw new ApiEnvelopeError({
      message: text || `HTTP ${response.status}`,
      status: response.status,
      errorCode: null,
      fieldErrors: null,
      retryable: response.status >= 500,
    });
  }

  if (!parsed || typeof parsed.ok !== "boolean") {
    trackClientTelemetry({
      route,
      userMode: mode,
      action,
      errorClass: "fatal",
      errorCode: null,
      message: `Invalid API response (HTTP ${response.status})`,
      status: response.status,
      requestId,
    });
    throw new ApiEnvelopeError({
      message: `Invalid API response (HTTP ${response.status})`,
      status: response.status,
      errorCode: null,
      fieldErrors: null,
      retryable: response.status >= 500,
    });
  }

  if (!response.ok || !parsed.ok) {
    trackClientTelemetry({
      route,
      userMode: mode,
      action,
      errorClass: classifyStatusError(response.status, parsed.retryable, parsed.errorCode),
      errorCode: parsed.errorCode,
      message: parsed.message || `HTTP ${response.status}`,
      status: response.status,
      requestId,
    });
    throw new ApiEnvelopeError({
      message: parsed.message || `HTTP ${response.status}`,
      status: response.status,
      errorCode: parsed.errorCode,
      fieldErrors: parsed.fieldErrors,
      retryable: parsed.retryable,
    });
  }

  if (shouldTrackSuccess(route)) {
    trackClientTelemetry({
      route,
      userMode: mode,
      action,
      outcome: "success",
      errorClass: "none",
      errorCode: null,
      message: null,
      status: response.status,
      requestId,
    });
  }

  return parsed.data as T;
}
