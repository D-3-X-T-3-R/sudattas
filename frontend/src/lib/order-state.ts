type StatusNameById = Map<string, string>;

export type ShipmentLike = {
  status?: string | null;
  shiprocketStatusId?: string | null;
  shiprocketStatusLabel?: string | null;
};

export type PaymentIntentLike = {
  status?: string | null;
};

function normalize(raw?: string | null): string {
  return (raw ?? "").trim().toLowerCase();
}

export function canonicalOrderStatusName(statusName?: string | null): string {
  const normalized = normalize(statusName);
  if (!normalized) return "unknown";
  if (normalized === "processing") return "processing order";
  return normalized;
}

export function statusNameFromId(
  statusId: string | undefined,
  statusNameById: StatusNameById
): string {
  if (!statusId) return "unknown";
  return canonicalOrderStatusName(statusNameById.get(statusId));
}

export function deriveOrderUiState(statusName?: string | null): string {
  const name = canonicalOrderStatusName(statusName);
  if (name.includes("cancel")) return "cancelled";
  if (name.includes("deliver")) return "delivered";
  if (name.includes("ship") || name.includes("transit")) return "shipped";
  if (name.includes("confirm") || name.includes("process")) return "processing";
  if (name.includes("pending") || name.includes("active_sale")) return "pending";
  return name;
}

export function normalizePaymentState(rawStatus?: string | null): string {
  const status = normalize(rawStatus);
  if (!status) return "pending";
  if (status.includes("needs_review")) return "needs_review";
  if (status.includes("refunded")) return "refunded";
  if (status.includes("captured") || status.includes("processed") || status.includes("paid")) {
    return "paid";
  }
  if (status.includes("failed")) return "failed";
  if (status.includes("verified")) return "verified";
  return status;
}

export function derivePaymentStateFromIntents(intents: PaymentIntentLike[]): string {
  if (!intents.length) return "not_started";
  const normalized = intents.map((intent) => normalizePaymentState(intent.status));
  if (normalized.includes("needs_review")) return "needs_review";
  if (normalized.includes("refunded")) return "refunded";
  if (normalized.includes("paid")) return "paid";
  if (normalized.includes("failed")) return "failed";
  if (normalized.includes("verified")) return "verified";
  return normalized[0] ?? "pending";
}

export function deriveShipmentState(shipments: ShipmentLike[]): string {
  if (!shipments.length) return "not_shipped";
  const mapByStatusId = (statusId?: string | null): string | null => {
    const id = (statusId ?? "").trim();
    if (id === "7" || id === "23") return "delivered";
    if (id === "8" || id === "9" || id === "10" || id === "14" || id === "15" || id === "16")
      return "issue";
    if (id === "17" || id === "38" || id === "56") return "out_for_delivery";
    if (id === "18" || id === "6" || id === "41" || id === "45" || id === "42")
      return "in_transit";
    return null;
  };

  for (const shipment of shipments) {
    if (mapByStatusId(shipment.shiprocketStatusId) === "delivered") return "delivered";
  }

  let best: string | null = null;
  for (const shipment of shipments) {
    const mapped = mapByStatusId(shipment.shiprocketStatusId);
    if (!mapped) continue;
    if (mapped === "issue") return "issue";
    if (!best) best = mapped;
  }
  if (best) return best;

  const labels = shipments
    .map((shipment) => normalize(shipment.shiprocketStatusLabel))
    .filter(Boolean);
  const statuses = shipments.map((shipment) => normalize(shipment.status)).filter(Boolean);
  if (labels.some((v) => v.includes("delivered")) || statuses.some((v) => v.includes("delivered"))) {
    return "delivered";
  }
  if (
    labels.some((v) => v.includes("cancel") || v.includes("rto")) ||
    statuses.some(
      (v) => v.includes("failed") || v.includes("returned") || v.includes("rto") || v.includes("cancel")
    )
  ) {
    return "issue";
  }
  if (
    labels.some((v) => v.includes("transit") || v.includes("delivery") || v.includes("awb")) ||
    statuses.some(
      (v) =>
        v.includes("shipped") ||
        v.includes("transit") ||
        v.includes("picked_up") ||
        v.includes("out_for_delivery") ||
        v.includes("awb")
    )
  ) {
    return "in_transit";
  }
  return statuses[0] ?? "pending";
}
