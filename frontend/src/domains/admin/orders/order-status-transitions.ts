/**
 * Mirrors `core_operations::order_state_machine::allowed_transitions` (DB status names).
 * Used to filter admin status dropdown so illegal transitions are not offered.
 */
const TARGETS_BY_CURRENT: Record<string, string[]> = {
  pending: ["pending", "confirmed", "needs_review", "cancelled"],
  confirmed: ["confirmed", "processing", "cancelled", "refunded"],
  processing: ["processing", "shipped", "cancelled", "refunded"],
  shipped: ["shipped", "delivered", "cancelled", "refunded"],
  delivered: ["delivered", "refunded"],
  needs_review: ["needs_review", "pending", "confirmed", "cancelled", "refunded"],
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
