import "server-only";

import { headers as nextHeaders } from "next/headers";

function readHeader(headers: Headers, name: string): string | null {
  const value = headers.get(name)?.trim();
  return value ? value : null;
}

function forwardedIpHeadersFromHeaders(headers: Headers): Record<string, string> {
  const xForwardedFor = readHeader(headers, "x-forwarded-for");
  const xRealIp = readHeader(headers, "x-real-ip");
  const cfConnectingIp = readHeader(headers, "cf-connecting-ip");

  const out: Record<string, string> = {};
  if (xForwardedFor) out["X-Forwarded-For"] = xForwardedFor;
  if (xRealIp) out["X-Real-Ip"] = xRealIp;
  if (!xRealIp && cfConnectingIp) out["X-Real-Ip"] = cfConnectingIp;
  return out;
}

export function forwardedIpHeadersFromRequest(request: Request): Record<string, string> {
  return forwardedIpHeadersFromHeaders(request.headers);
}

export async function forwardedIpHeadersFromCurrentRequest(): Promise<Record<string, string>> {
  try {
    const h = await nextHeaders();
    return forwardedIpHeadersFromHeaders(h);
  } catch {
    return {};
  }
}
