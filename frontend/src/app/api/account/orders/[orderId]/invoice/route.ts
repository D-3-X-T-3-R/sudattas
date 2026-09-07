import {
  apiError,
  callGraphqlAsCustomer,
  graphqlErrorToApiStatus,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";
import { graphqlBaseUrl, serverEnv } from "@/lib/env/server";

const ORDER_INVOICE_DOWNLOAD_QUERY = `query AccountOrderInvoiceDownload($orderId: String!) {
  getOrderInvoiceDownload(orderId: $orderId) {
    downloadUrl
  }
}`;

type InvoiceDownloadRow = {
  downloadUrl: string;
};

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
  if (!["http:", "https:"].includes(parsed.protocol)) {
    return null;
  }

  // Guard against SSRF: only allow invoice URLs from the configured GraphQL/API origin.
  const allowedOrigin = new URL(graphqlBaseUrl()).origin;
  if (parsed.origin !== allowedOrigin) {
    return null;
  }

  return parsed;
}

export async function GET(
  _request: Request,
  context: { params: Promise<{ orderId: string }> }
) {
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const { orderId } = await context.params;
  const trimmedOrderId = orderId.trim();
  if (!trimmedOrderId) {
    return apiError("Order ID is required", 400, "VALIDATION_ERROR");
  }

  const result = await callGraphqlAsCustomer<{
    getOrderInvoiceDownload?: InvoiceDownloadRow | null;
  }>(userId, ORDER_INVOICE_DOWNLOAD_QUERY, { orderId: trimmedOrderId });

  if (result.errors?.length) {
    const { status, message } = graphqlErrorToApiStatus(result.errors, "Failed to fetch invoice");
    const normalized = message.toLowerCase();
    if (
      normalized.includes("forbidden") ||
      normalized.includes("not found for current user")
    ) {
      return apiError("Access denied", 403, "FORBIDDEN");
    }
    if (normalized.includes("not found")) {
      return apiError("Invoice not found", 404, "NOT_FOUND");
    }
    return apiError(message, status, "GRAPHQL_ERROR");
  }

  const invoice = result.data?.getOrderInvoiceDownload;
  const downloadUrl = normalizeDownloadUrl(invoice?.downloadUrl ?? "");
  if (!downloadUrl) {
    return apiError("Invoice not found", 404, "NOT_FOUND");
  }

  const internalSecret = serverEnv.INTERNAL_API_SECRET?.trim();
  if (!internalSecret) {
    return apiError("Invoice download is not configured", 500, "SERVER_CONFIG_ERROR");
  }

  let upstream: Response;
  try {
    upstream = await fetch(downloadUrl.toString(), {
      method: "GET",
      headers: {
        "X-Internal-Auth": internalSecret,
        "X-Customer-User-Id": userId,
      },
      cache: "no-store",
    });
  } catch {
    return apiError("Invoice service is unavailable", 503, "UPSTREAM_UNAVAILABLE");
  }

  if (!upstream.ok) {
    if (upstream.status === 403) {
      return apiError("Access denied", 403, "FORBIDDEN");
    }
    if (upstream.status === 404) {
      return apiError("Invoice not found", 404, "NOT_FOUND");
    }
    return apiError("Failed to download invoice", 502, "UPSTREAM_ERROR");
  }

  const contentType = upstream.headers.get("content-type")?.trim() || "application/pdf";
  const upstreamDisposition = upstream.headers.get("content-disposition")?.trim() ?? "";
  const fallbackName =
    downloadUrl.pathname.split("/").filter(Boolean).at(-2) ?? "invoice";
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
