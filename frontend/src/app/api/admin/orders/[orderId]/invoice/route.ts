import { getAdminSession } from "@/lib/admin-auth-server";
import { graphQlUrl } from "@/lib/server-session-auth";
import { graphqlBaseUrl } from "@/lib/env/server";

/**
 * Admin equivalent of app/api/account/orders/[orderId]/invoice/route.ts — same SSRF-safe
 * download-then-proxy shape, but authenticated as the admin session (Bearer JWT, checked against
 * the backend's admin allowlist) rather than the internal-service secret + customer id used for
 * self-service downloads. get_order_invoice_download skips the ownership check entirely when
 * context.is_admin() (backend/graphql/src/resolvers/invoices/handlers.rs), so any order works.
 */

const INVOICE_QUERY = `query AdminOrderInvoiceDownload($orderId: String!) {
  getOrderInvoiceDownload(orderId: $orderId) { downloadUrl }
}`;

function sanitizeFileName(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) return "invoice.pdf";
  const safe = trimmed.replace(/[^a-zA-Z0-9._-]/g, "_");
  return safe.toLowerCase().endsWith(".pdf") ? safe : `${safe}.pdf`;
}

function normalizeDownloadUrl(raw: string): URL | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;
  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    return null;
  }
  if (!["http:", "https:"].includes(parsed.protocol)) return null;
  // Guard against SSRF: only allow invoice URLs from the configured GraphQL/API origin.
  const allowedOrigin = new URL(graphqlBaseUrl()).origin;
  if (parsed.origin !== allowedOrigin) return null;
  return parsed;
}

export async function GET(
  _request: Request,
  context: { params: Promise<{ orderId: string }> }
) {
  const session = await getAdminSession();
  const token = session?.idToken ?? session?.accessToken;
  if (!token) {
    return Response.json({ ok: false, message: "Unauthorized" }, { status: 401 });
  }
  const authHeader = token.startsWith("Bearer ") ? token : `Bearer ${token}`;

  const { orderId } = await context.params;
  const trimmedOrderId = orderId.trim();
  if (!trimmedOrderId) {
    return Response.json({ ok: false, message: "Order ID is required" }, { status: 400 });
  }

  let graphqlRes: Response;
  try {
    graphqlRes = await fetch(graphQlUrl(), {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: authHeader },
      body: JSON.stringify({ query: INVOICE_QUERY, variables: { orderId: trimmedOrderId } }),
      cache: "no-store",
    });
  } catch {
    return Response.json({ ok: false, message: "GraphQL service unavailable" }, { status: 502 });
  }

  const parsed = (await graphqlRes.json().catch(() => null)) as {
    data?: { getOrderInvoiceDownload?: { downloadUrl?: string } };
    errors?: Array<{ message?: string }>;
  } | null;

  if (parsed?.errors?.length) {
    const message = parsed.errors[0]?.message ?? "Failed to fetch invoice";
    const status = message.toLowerCase().includes("not found") ? 404 : 400;
    return Response.json({ ok: false, message }, { status });
  }

  const downloadUrl = normalizeDownloadUrl(
    parsed?.data?.getOrderInvoiceDownload?.downloadUrl ?? ""
  );
  if (!downloadUrl) {
    return Response.json({ ok: false, message: "Invoice not found" }, { status: 404 });
  }

  let upstream: Response;
  try {
    upstream = await fetch(downloadUrl.toString(), {
      method: "GET",
      headers: { Authorization: authHeader },
      cache: "no-store",
    });
  } catch {
    return Response.json({ ok: false, message: "Invoice service is unavailable" }, { status: 503 });
  }

  if (!upstream.ok) {
    const status = upstream.status === 404 ? 404 : upstream.status === 403 ? 403 : 502;
    return Response.json(
      { ok: false, message: status === 404 ? "Invoice not found" : "Failed to download invoice" },
      { status }
    );
  }

  const contentType = upstream.headers.get("content-type")?.trim() || "application/pdf";
  const upstreamDisposition = upstream.headers.get("content-disposition")?.trim() ?? "";
  const fallbackName = downloadUrl.pathname.split("/").filter(Boolean).at(-2) ?? "invoice";
  const fileName = sanitizeFileName(`Invoice_${fallbackName}.pdf`);
  const contentDisposition = upstreamDisposition || `attachment; filename="${fileName}"`;

  const body = await upstream.arrayBuffer();
  return new Response(body, {
    status: 200,
    headers: {
      "Content-Type": contentType,
      "Content-Disposition": contentDisposition,
      "Cache-Control": "private, no-store",
    },
  });
}
