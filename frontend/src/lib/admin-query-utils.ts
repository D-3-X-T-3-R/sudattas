/** Shared pagination helpers used by both admin-queries.ts and admin-product-queries.ts. */

export const ORDER_PAGE_SIZE = 50;
export const PRODUCT_PAGE_SIZE = 50;

export function toPositiveInt(value: string | undefined, fallback: number): number {
  if (!value) return fallback;
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed < 1) return fallback;
  return parsed;
}

export function normalizeLimit(limit: string | undefined): string {
  const n = toPositiveInt(limit, ORDER_PAGE_SIZE);
  return String(Math.min(n, ORDER_PAGE_SIZE));
}

export function normalizeOffset(offset: string | undefined): string | undefined {
  if (!offset) return undefined;
  const parsed = Number.parseInt(offset, 10);
  if (!Number.isFinite(parsed) || parsed < 0) return "0";
  return String(parsed);
}
