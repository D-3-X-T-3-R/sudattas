/**
 * Mirrors `core_operations::order_state_machine::allowed_transitions` (DB status names) — minus
 * "cancelled"/"partially_cancelled" as raw picks. Those go through the dedicated "Cancel order"
 * action (`deleteOrder`) instead, which — unlike a raw status flip via `updateOrder` — enforces
 * the cancel window, checks fulfillment status, and restores inventory. Offering "cancelled" here
 * would let an admin bypass all of that with a plain status-dropdown pick.
 */
const TARGETS_BY_CURRENT: Record<string, string[]> = {
  pending: ["pending", "confirmed", "needs_review"],
  confirmed: ["confirmed", "processing", "refunded"],
  processing: ["processing", "shipped", "refunded"],
  shipped: ["shipped", "delivered", "refunded"],
  delivered: ["delivered", "refunded"],
  needs_review: ["needs_review", "pending", "confirmed", "refunded"],
  cancelled: ["cancelled"],
  refunded: ["refunded"],
};

function normalizeStatusName(name: string): string {
  return name.trim().toLowerCase();
}

/**
 * Lowercase status names allowed as the next status (including staying on current).
 * `null` if current name is unknown — caller should show all statuses and rely on server.
 */
export function allowedTargetStatusNames(currentStatusName: string): string[] | null {
  const key = normalizeStatusName(currentStatusName);
  return TARGETS_BY_CURRENT[key] ?? null;
}

export function filterStatusesForTransition<T extends { statusId: string; statusName: string }>(
  statuses: T[],
  currentStatusName: string
): T[] {
  const allowed = allowedTargetStatusNames(currentStatusName);
  if (allowed == null) {
    return statuses;
  }
  const allowedSet = new Set(allowed);
  return statuses.filter((s) => allowedSet.has(normalizeStatusName(s.statusName)));
}
