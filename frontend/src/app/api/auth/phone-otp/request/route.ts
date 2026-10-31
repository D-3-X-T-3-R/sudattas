import { NextResponse } from "next/server";

/* ── Phone/OTP sign-in — disabled for now, kept for a possible future re-enable ──
import { graphqlBaseUrl } from "@/lib/env/server";
import { publicEnv } from "@/lib/env/public";

type RequestPayload = {
  phone?: string;
};

export async function POST(request: Request) {
  let body: RequestPayload;
  try {
    body = (await request.json()) as RequestPayload;
  } catch {
    return NextResponse.json(
      { ok: false, error: "Invalid JSON payload" },
      { status: 400 }
    );
  }

  const phone = body.phone?.trim() ?? "";
  if (!phone) {
    return NextResponse.json(
      { ok: false, error: "Phone number is required" },
      { status: 400 }
    );
  }

  const channel = publicEnv.NEXT_PUBLIC_PHONE_OTP_CHANNEL;
  const response = await fetch(`${graphqlBaseUrl()}/auth/phone-otp/request`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      phone,
      ...(channel ? { channel } : {}),
    }),
    cache: "no-store",
  });

  const json = (await response.json().catch(() => ({}))) as {
    ok?: boolean;
    error?: string;
  };
  if (!response.ok || json.ok !== true) {
    return NextResponse.json(
      { ok: false, error: json.error ?? "Failed to send OTP" },
      { status: response.status || 502 }
    );
  }

  return NextResponse.json({ ok: true });
}
──────────────────────────────────────────────────────────────────────────── */

export async function POST() {
  return NextResponse.json(
    { ok: false, error: "Phone sign-in is currently disabled" },
    { status: 410 }
  );
}
