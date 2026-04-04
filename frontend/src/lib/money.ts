const INR_FORMATTER = new Intl.NumberFormat("en-IN", {
  style: "currency",
  currency: "INR",
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

export const MAX_MONEY_PAISE = 2_147_483_647;
export const MAX_MONEY_RUPEES = MAX_MONEY_PAISE / 100;
export const RUPEES_INPUT_REGEX = /^\d+(\.\d{1,2})?$/;

export function parsePaise(value: string | number | null | undefined): number {
  if (typeof value === "number") {
    return Number.isFinite(value) ? Math.trunc(value) : 0;
  }
  const n = Number.parseInt(String(value ?? "").trim(), 10);
  return Number.isFinite(n) ? n : 0;
}

export function paiseToRupeesNumber(value: string | number | null | undefined): number {
  return parsePaise(value) / 100;
}

export function formatInrFromPaise(value: string | number | null | undefined): string {
  return INR_FORMATTER.format(paiseToRupeesNumber(value));
}

/** Convert rupees input like `499.00` to integer paise without float rounding. */
export function rupeesInputToPaise(rupeesInput: string): number {
  const s = (rupeesInput || "0").trim();
  if (!RUPEES_INPUT_REGEX.test(s)) return 0;
  const parts = s.split(".");
  const major = Number.parseInt(parts[0] || "0", 10) || 0;
  const minorStr = (parts[1] || "00").slice(0, 2).padEnd(2, "0");
  const minor = Number.parseInt(minorStr, 10) || 0;
  return major * 100 + minor;
}

/** Parse optional rupee input into paise; returns null for empty/invalid values. */
export function optionalRupeesInputToPaise(rupeesInput: string): number | null {
  const s = (rupeesInput || "").trim();
  if (!s) return null;
  if (!RUPEES_INPUT_REGEX.test(s)) return null;
  return rupeesInputToPaise(s);
}

export function paiseToRupeesInput(value: string | number | null | undefined): string {
  return paiseToRupeesNumber(value).toFixed(2);
}
