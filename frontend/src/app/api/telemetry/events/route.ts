import { NextResponse } from "next/server";
import { z } from "zod";
import { getAdminSession } from "@/lib/admin-auth-server";
import { appendTelemetryEvent, listTelemetryEvents } from "@/lib/telemetry-store";

const telemetryEventSchema = z.object({
  occurredAt: z.string().datetime().optional(),
  route: z.string().min(1).max(200),
  pageRoute: z.string().max(200).nullable().optional(),
  userMode: z.enum(["public", "account", "admin"]),
  action: z.string().min(1).max(240),
  outcome: z.enum(["success", "failure"]).optional(),
  errorClass: z.enum([
    "none",
    "unauthorized",
    "validation",
    "retryable",
    "network",
    "fatal",
    "boundary",
  ]),
  errorCode: z.string().max(120).nullable().optional(),
  message: z.string().max(1000).nullable().optional(),
  status: z.number().int().min(100).max(599).nullable().optional(),
  requestId: z.string().max(200).nullable().optional(),
  online: z.boolean().nullable().optional(),
  userAgent: z.string().max(1000).nullable().optional(),
  effectiveType: z.string().max(50).nullable().optional(),
  downlink: z.number().nonnegative().max(10_000).nullable().optional(),
  rtt: z.number().nonnegative().max(600_000).nullable().optional(),
});

function badRequest(message: string) {
  return NextResponse.json(
    {
      ok: false,
      data: null,
      errorCode: "VALIDATION_ERROR",
      message,
      fieldErrors: null,
      retryable: false,
    },
    { status: 400 }
  );
}

export async function POST(request: Request) {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return badRequest("Invalid telemetry JSON payload");
  }

  const parsed = telemetryEventSchema.safeParse(body);
  if (!parsed.success) {
    return badRequest(parsed.error.issues[0]?.message ?? "Invalid telemetry payload");
  }

  const payload = parsed.data;
  await appendTelemetryEvent({
    occurredAt: payload.occurredAt ?? new Date().toISOString(),
    route: payload.route,
    pageRoute: payload.pageRoute ?? null,
    userMode: payload.userMode,
    action: payload.action,
    outcome: payload.outcome ?? "failure",
    errorClass: payload.errorClass,
    errorCode: payload.errorCode ?? null,
    message: payload.message ?? null,
    status: payload.status ?? null,
    requestId: payload.requestId ?? null,
    online: payload.online ?? null,
    userAgent: payload.userAgent ?? null,
    effectiveType: payload.effectiveType ?? null,
    downlink: payload.downlink ?? null,
    rtt: payload.rtt ?? null,
  });

  return NextResponse.json({
    ok: true,
    data: { accepted: true },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}

export async function GET(request: Request) {
  const admin = await getAdminSession();
  if (!admin) {
    return NextResponse.json(
      {
        ok: false,
        data: null,
        errorCode: "UNAUTHORIZED",
        message: "Unauthorized telemetry access",
        fieldErrors: null,
        retryable: false,
      },
      { status: 401 }
    );
  }

  const { searchParams } = new URL(request.url);
  const limitRaw = searchParams.get("limit") ?? "100";
  const limitParsed = Number.parseInt(limitRaw, 10);
  const limit = Number.isFinite(limitParsed) ? Math.max(1, Math.min(limitParsed, 500)) : 100;
  const mode = searchParams.get("mode");
  const errorClass = searchParams.get("errorClass");
  const outcome = searchParams.get("outcome");
  const errorCode = searchParams.get("errorCode");
  const routeContains = searchParams.get("q");

  const events = await listTelemetryEvents({
    limit,
    userMode:
      mode === "public" || mode === "account" || mode === "admin"
        ? mode
        : undefined,
    outcome: outcome === "success" || outcome === "failure" ? outcome : undefined,
    errorClass:
      errorClass === "none" ||
      errorClass === "unauthorized" ||
      errorClass === "validation" ||
      errorClass === "retryable" ||
      errorClass === "network" ||
      errorClass === "fatal" ||
      errorClass === "boundary"
        ? errorClass
        : undefined,
    errorCode: errorCode || undefined,
    routeContains: routeContains || undefined,
  });

  return NextResponse.json({
    ok: true,
    data: {
      events,
      filters: {
        limit,
        mode: mode ?? null,
        outcome: outcome ?? null,
        errorClass: errorClass ?? null,
        errorCode: errorCode ?? null,
        q: routeContains ?? null,
      },
    },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
