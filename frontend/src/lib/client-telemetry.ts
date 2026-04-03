"use client";

export type TelemetryUserMode = "public" | "account" | "admin";
export type TelemetryErrorClass =
  | "none"
  | "unauthorized"
  | "validation"
  | "retryable"
  | "network"
  | "fatal"
  | "boundary";

export type ClientTelemetryEvent = {
  route: string;
  userMode: TelemetryUserMode;
  action: string;
  outcome?: "success" | "failure";
  errorClass: TelemetryErrorClass;
  errorCode?: string | null;
  message?: string | null;
  status?: number | null;
  requestId?: string | null;
  occurredAt?: string;
};

function pathToMode(pathname: string): TelemetryUserMode {
  if (pathname.startsWith("/api/admin") || pathname.startsWith("/imtheboss")) {
    return "admin";
  }
  if (
    pathname.startsWith("/api/account") ||
    pathname.startsWith("/api/checkout") ||
    pathname.startsWith("/profile") ||
    pathname.startsWith("/checkout")
  ) {
    return "account";
  }
  return "public";
}

function networkContext() {
  if (typeof navigator === "undefined") {
    return { online: null, userAgent: null, effectiveType: null, downlink: null, rtt: null };
  }
  const nav = navigator as Navigator & {
    connection?: { effectiveType?: string; downlink?: number; rtt?: number };
  };
  return {
    online: typeof nav.onLine === "boolean" ? nav.onLine : null,
    userAgent: nav.userAgent ?? null,
    effectiveType: nav.connection?.effectiveType ?? null,
    downlink: typeof nav.connection?.downlink === "number" ? nav.connection.downlink : null,
    rtt: typeof nav.connection?.rtt === "number" ? nav.connection.rtt : null,
  };
}

function dedupeKey(event: ClientTelemetryEvent): string {
  return [
    event.route,
    event.userMode,
    event.action,
    event.errorClass,
    event.errorCode ?? "",
    event.status ?? "",
    event.message ?? "",
  ].join("|");
}

const recentlySent = new Map<string, number>();
const DEDUPE_WINDOW_MS = 10_000;

export function trackClientTelemetry(event: ClientTelemetryEvent): void {
  if (typeof window === "undefined") return;
  if (event.route.startsWith("/api/telemetry/events")) return;

  const now = Date.now();
  const key = dedupeKey(event);
  const previousTs = recentlySent.get(key);
  if (previousTs && now - previousTs < DEDUPE_WINDOW_MS) return;
  recentlySent.set(key, now);

  for (const [k, ts] of recentlySent) {
    if (now - ts > DEDUPE_WINDOW_MS * 2) recentlySent.delete(k);
  }

  const payload = {
    occurredAt: event.occurredAt ?? new Date().toISOString(),
    route: event.route,
    pageRoute: window.location.pathname,
    userMode: event.userMode || pathToMode(event.route),
    action: event.action,
    outcome: event.outcome ?? "failure",
    errorClass: event.errorClass,
    errorCode: event.errorCode ?? null,
    message: event.message ?? null,
    status: typeof event.status === "number" ? event.status : null,
    requestId: event.requestId ?? null,
    ...networkContext(),
  };

  try {
    const body = JSON.stringify(payload);
    if (typeof navigator.sendBeacon === "function") {
      const blob = new Blob([body], { type: "application/json" });
      navigator.sendBeacon("/api/telemetry/events", blob);
      return;
    }
    void fetch("/api/telemetry/events", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
      keepalive: true,
    });
  } catch {
    // Telemetry is best-effort only.
  }
}

export function classifyStatusError(
  status: number,
  retryable: boolean,
  errorCode: string | null
): TelemetryErrorClass {
  if (status === 401 || status === 403 || errorCode === "UNAUTHORIZED") return "unauthorized";
  if (status === 400 || errorCode === "VALIDATION_ERROR" || errorCode === "BAD_REQUEST") {
    return "validation";
  }
  if (status === 429 || retryable || status >= 500) return "retryable";
  return "fatal";
}
