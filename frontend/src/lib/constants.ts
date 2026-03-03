export const INR = new Intl.NumberFormat("en-IN", {
  style: "currency",
  currency: "INR",
  maximumFractionDigits: 0,
});

export const COLLECTIONS = [
  { key: "Signature", blurb: "Iconic borders. Clean bodies." },
  { key: "Studio", blurb: "Modern art on fabric." },
  { key: "Everyday Luxury", blurb: "Light, sharp, repeatable." },
  { key: "Occasion", blurb: "Wedding and festive statements." },
] as const;
