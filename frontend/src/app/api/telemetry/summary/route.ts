import { NextResponse } from "next/server";
import { getAdminSession } from "@/lib/admin-auth-server";
import { listTelemetryEvents, type StoredTelemetryEvent } from "@/lib/telemetry-store";
import { coreOpsMetricsUrl, graphqlMetricsUrl } from "@/lib/env/server";

type Ratio = { numerator: number; denominator: number; percent: number };

function ratio(numerator: number, denominator: number): Ratio {
  const safeDen = denominator <= 0 ? 0 : denominator;
  const pct = safeDen === 0 ? 0 : (numerator / safeDen) * 100;
  return { numerator, denominator: safeDen, percent: Number(pct.toFixed(2)) };
}

function withinWindow(events: StoredTelemetryEvent[], hours: number): StoredTelemetryEvent[] {
  const now = Date.now();
  const cutoff = now - hours * 60 * 60 * 1000;
  return events.filter((e) => Date.parse(e.occurredAt) >= cutoff);
}

function releaseConfidenceScore(input: {
  checkoutFailureRate: number;
  adminFailureRate: number;
  paymentMismatchRate: number;
  loginFailureRate: number;
}): number {
  const score =
    100 -
    input.checkoutFailureRate * 1.4 -
    input.adminFailureRate * 1.2 -
    input.paymentMismatchRate * 1.6 -
    input.loginFailureRate * 0.8;
  return Math.max(0, Math.min(100, Number(score.toFixed(1))));
}

type PromSample = { name: string; labels: Record<string, string>; value: number };

function parsePrometheus(text: string): PromSample[] {
  const out: PromSample[] = [];
  for (const line of text.split(/\r?\n/)) {
    const t = line.trim();
    if (!t || t.startsWith("#")) continue;
    const match = t.match(
      /^([a-zA-Z_:][a-zA-Z0-9_:]*)(\{([^}]*)\})?\s+([-+]?[0-9]*\.?[0-9]+(?:[eE][-+]?[0-9]+)?)$/
    );
    if (!match) continue;
    const name = match[1];
    const labelsRaw = match[3] ?? "";
    const value = Number.parseFloat(match[4]);
    if (!Number.isFinite(value)) continue;
    const labels: Record<string, string> = {};
    for (const part of labelsRaw.split(",")) {
      const p = part.trim();
      if (!p) continue;
      const m = p.match(/^([a-zA-Z_][a-zA-Z0-9_]*)="(.*)"$/);
      if (!m) continue;
      labels[m[1]] = m[2];
    }
    out.push({ name, labels, value });
  }
  return out;
}

function sumMetric(
  samples: PromSample[],
  name: string,
  labels?: Record<string, string>
): number {
  return samples
    .filter((s) => {
      if (s.name !== name) return false;
      if (!labels) return true;
      return Object.entries(labels).every(([k, v]) => s.labels[k] === v);
    })
    .reduce((acc, s) => acc + s.value, 0);
}

function combineRatios(a: Ratio, b: Ratio): Ratio {
  return ratio(a.numerator + b.numerator, a.denominator + b.denominator);
}

export async function GET() {
  const admin = await getAdminSession();
  if (!admin) {
    return NextResponse.json(
      {
        ok: false,
        data: null,
        errorCode: "UNAUTHORIZED",
        message: "Unauthorized telemetry summary access",
        fieldErrors: null,
        retryable: false,
      },
      { status: 401 }
    );
  }

  const recent = withinWindow(await listTelemetryEvents({ limit: 5000 }), 24);

  const loginEvents = recent.filter((e) => e.action.startsWith("AUTH_SIGN_IN_"));
  const loginFailures = loginEvents.filter((e) => e.outcome === "failure").length;
  const loginRatio = ratio(loginFailures, loginEvents.length);

  const browseSuccess = recent.filter(
    (e) => e.route === "/api/products" && e.outcome === "success"
  ).length;
  const checkoutStarted = recent.filter(
    (e) => e.route === "/api/checkout/place-order" && e.outcome === "success"
  ).length;
  const conversionDropoff = ratio(
    Math.max(0, browseSuccess - checkoutStarted),
    Math.max(1, browseSuccess)
  );

  const checkoutEvents = recent.filter((e) => e.route.startsWith("/api/checkout/"));
  const checkoutFailures = checkoutEvents.filter((e) => e.outcome === "failure").length;
  const checkoutFailureRatio = ratio(checkoutFailures, checkoutEvents.length);

  const paymentVerifyEvents = recent.filter(
    (e) => e.route === "/api/checkout/verify-payment"
  );
  const paymentMismatchFailures = paymentVerifyEvents.filter((e) => {
    if (e.outcome !== "failure") return false;
    const msg = (e.message ?? "").toLowerCase();
    const code = (e.errorCode ?? "").toLowerCase();
    return msg.includes("mismatch") || code.includes("mismatch");
  }).length;
  const paymentMismatchRatio = ratio(
    paymentMismatchFailures,
    Math.max(1, paymentVerifyEvents.length)
  );

  const adminEvents = recent.filter((e) => e.route.startsWith("/api/admin/"));
  const adminFailures = adminEvents.filter((e) => e.outcome === "failure").length;
  const adminFailureRatio = ratio(adminFailures, adminEvents.length);

  let backendMetricsAvailable = false;
  let backendCheckoutRatio = ratio(0, 0);
  let backendPaymentMismatchRatio = ratio(0, 0);
  let backendAdminFailureRatio = ratio(0, 0);
  let backendAuthFailureCount = 0;
  let webhookLatencyAverageMs: number | null = null;

  try {
    const [graphqlMetricsText, coreOpsMetricsText] = await Promise.all([
      fetch(graphqlMetricsUrl(), { cache: "no-store" }).then((r) => r.text()),
      fetch(coreOpsMetricsUrl(), { cache: "no-store" }).then((r) => r.text()),
    ]);

    const gql = parsePrometheus(graphqlMetricsText);
    const ops = parsePrometheus(coreOpsMetricsText);
    backendMetricsAvailable = gql.length > 0 || ops.length > 0;

    const placeOrderOk = sumMetric(gql, "place_order_total", { outcome: "ok" });
    const placeOrderErr = sumMetric(gql, "place_order_total", { outcome: "error" });
    backendCheckoutRatio = ratio(placeOrderErr, placeOrderOk + placeOrderErr);

    const paymentMismatch = sumMetric(ops, "payment_mismatch_total");
    const webhookAccepted = sumMetric(gql, "webhook_accepted_total");
    backendPaymentMismatchRatio = ratio(paymentMismatch, webhookAccepted);

    const adminDenied = sumMetric(gql, "graphql_admin_authz_denied_total");
    const adminReqOk = sumMetric(gql, "graphql_requests_total", { outcome: "ok" });
    backendAdminFailureRatio = ratio(adminDenied, adminDenied + adminReqOk);

    backendAuthFailureCount = sumMetric(gql, "graphql_auth_rejection_total", {
      kind: "unauthorized",
    });

    const whSum = sumMetric(ops, "webhook_processing_duration_seconds_sum");
    const whCount = sumMetric(ops, "webhook_processing_duration_seconds_count");
    if (whCount > 0) {
      webhookLatencyAverageMs = Number(((whSum / whCount) * 1000).toFixed(2));
    }
  } catch {
    backendMetricsAvailable = false;
  }

  const checkoutCombined = combineRatios(checkoutFailureRatio, backendCheckoutRatio);
  const paymentMismatchCombined = combineRatios(
    paymentMismatchRatio,
    backendPaymentMismatchRatio
  );
  const adminCombined = combineRatios(adminFailureRatio, backendAdminFailureRatio);

  const confidence = releaseConfidenceScore({
    checkoutFailureRate: checkoutCombined.percent,
    adminFailureRate: adminCombined.percent,
    paymentMismatchRate: paymentMismatchCombined.percent,
    loginFailureRate: loginRatio.percent,
  });

  return NextResponse.json({
    ok: true,
    data: {
      windowHours: 24,
      loginFailureRate: loginRatio,
      cartConversionDropoff: conversionDropoff,
      checkoutFailureRate: checkoutCombined,
      paymentMismatchRate: paymentMismatchCombined,
      adminActionFailureRate: adminCombined,
      releaseConfidence: { score: confidence, scale: "0-100" },
      webhookProcessingLatency: {
        available: backendMetricsAvailable && webhookLatencyAverageMs !== null,
        averageMs: webhookLatencyAverageMs,
        message:
          backendMetricsAvailable && webhookLatencyAverageMs !== null
            ? null
            : "Backend webhook latency metrics unavailable.",
      },
      backendSignals: {
        available: backendMetricsAvailable,
        authUnauthorizedCount: backendAuthFailureCount,
      },
    },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
