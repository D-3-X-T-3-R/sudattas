import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

const ORDER_INVOICE_DOWNLOAD_QUERY = `query AccountOrderInvoiceDownload($orderId: String!) {
  getOrderInvoiceDownload(orderId: $orderId) {
    invoiceId
    invoiceNumber
    orderId
    generatedAt
    fileName
    contentType
    pdfBase64
  }
}`;

type InvoiceDownloadRow = {
  invoiceId: string;
  invoiceNumber: string;
  orderId: string;
  generatedAt: string;
  fileName: string;
  contentType: string;
  pdfBase64: string;
};

function sanitizeFileName(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) return "invoice.pdf";
  const safe = trimmed.replace(/[^a-zA-Z0-9._-]/g, "_");
  return safe.toLowerCase().endsWith(".pdf") ? safe : `${safe}.pdf`;
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
    const message = result.errors[0]?.message ?? "Failed to fetch invoice";
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
    return apiError(message, 400, "GRAPHQL_ERROR");
  }

  const invoice = result.data?.getOrderInvoiceDownload;
  if (!invoice?.pdfBase64) {
    return apiError("Invoice not found", 404, "NOT_FOUND");
  }

  let pdfBody: ArrayBuffer;
  try {
    const decoded = Buffer.from(invoice.pdfBase64, "base64");
    pdfBody = decoded.buffer.slice(
      decoded.byteOffset,
      decoded.byteOffset + decoded.byteLength
    );
  } catch {
    return apiError("Invoice payload is corrupted", 500, "INVOICE_DECODE_ERROR");
  }

  const fileName = sanitizeFileName(invoice.fileName || `${invoice.invoiceNumber}.pdf`);
  const contentType = invoice.contentType?.trim() || "application/pdf";

  return new Response(pdfBody, {
    status: 200,
    headers: {
      "Content-Type": contentType,
      "Content-Disposition": `attachment; filename="${fileName}"`,
      "Cache-Control": "private, no-store",
    },
  });
}
