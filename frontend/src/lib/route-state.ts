import { ApiEnvelopeError } from "@/lib/api-envelope";

export type RouteMode = "public" | "account" | "admin";
export type RouteFailureKind =
  | "unauthorized"
  | "stale"
  | "retryable"
  | "fatal";

export type RouteFailureUi = {
  kind: RouteFailureKind;
  title: string;
  message: string;
};

function looksLikeStaleSessionMessage(message: string): boolean {
  const m = message.toLowerCase();
  return (
    m.includes("session invalid") ||
    m.includes("session not found") ||
    m.includes("expired")
  );
}

function looksLikeUnauthorizedMessage(message: string): boolean {
  const m = message.toLowerCase();
  return m.includes("unauthorized") || m.includes("forbidden");
}

function classifyFailure(error: unknown): RouteFailureKind {
  if (error instanceof ApiEnvelopeError) {
    if (error.status === 401 || error.status === 403) return "unauthorized";
    if (looksLikeStaleSessionMessage(error.message)) return "stale";
    if (error.status === 429 || error.retryable || error.status >= 500) {
      return "retryable";
    }
    return "fatal";
  }
  if (error instanceof Error) {
    if (looksLikeUnauthorizedMessage(error.message)) return "unauthorized";
    if (looksLikeStaleSessionMessage(error.message)) return "stale";
    const m = error.message.toLowerCase();
    if (
      m.includes("network") ||
      m.includes("timeout") ||
      m.includes("fetch failed") ||
      m.includes("too many requests")
    ) {
      return "retryable";
    }
  }
  return "fatal";
}

export function toRouteFailureUi(
  mode: RouteMode,
  error: unknown
): RouteFailureUi {
  const kind = classifyFailure(error);
  if (mode === "public") {
    if (kind === "unauthorized" || kind === "stale") {
      return {
        kind,
        title: "Session expired",
        message: "Please refresh and continue shopping.",
      };
    }
    if (kind === "retryable") {
      return {
        kind,
        title: "Temporary issue",
        message: "We could not load this right now. Please try again in a moment.",
      };
    }
    return {
      kind,
      title: "Something went wrong",
      message: "Please try again shortly.",
    };
  }

  if (mode === "account") {
    if (kind === "unauthorized" || kind === "stale") {
      return {
        kind,
        title: "Sign in required",
        message: "Please sign in again to access your account details.",
      };
    }
    if (kind === "retryable") {
      return {
        kind,
        title: "Temporary issue",
        message: "We could not load your account data right now. Please retry.",
      };
    }
    return {
      kind,
      title: "Could not load account data",
      message: "Please try again later.",
    };
  }

  if (kind === "unauthorized" || kind === "stale") {
    return {
      kind,
      title: "Admin access required",
      message: "Your admin session is missing or expired. Sign in again.",
    };
  }
  if (kind === "retryable") {
    return {
      kind,
      title: "Temporary admin issue",
      message: "This admin data is temporarily unavailable. Try again.",
    };
  }
  return {
    kind,
    title: "Could not load admin data",
    message: "Please try again later.",
  };
}
