import type { Product } from "@/lib/schemas";

const categoryHero =
  "https://images.unsplash.com/photo-1585916420730-d7f95e942d43?auto=format&fit=crop&w=1200&q=80";
const heroOne =
  "https://images.unsplash.com/photo-1585916420730-d7f95e942d43?auto=format&fit=crop&w=1600&q=80";
const heroTwo =
  "https://images.unsplash.com/photo-1594938298603-c8148c4dae35?auto=format&fit=crop&w=1600&q=80";

export const HERO_SLIDES = [
  {
    eyebrow: "NEW DROP",
    title: "Sudatta's Signature Sarees",
    cta: "Discover the collection",
    image: heroOne,
    imageAlt: "Editorial fashion hero image",
    tone: "dark" as const,
  },
  {
    eyebrow: "CRAFT • MODERN",
    title: "Borders that feel like jewellery",
    cta: "Shop best sellers",
    image: heroTwo,
    imageAlt: "Luxury editorial fabric detail",
    tone: "dark" as const,
  },
];

export const COLLECTION_IMAGES = [
  heroOne,
  heroTwo,
  categoryHero,
  heroOne,
];

export const PRODUCTS_SEED: Product[] = [
  {
    id: "p1",
    name: "Ivory Silk with Folk Border",
    collection: "Signature",
    price: 6499,
    rating: 4.7,
    reviews: 312,
    fabric: "Silk Blend",
    occasion: "Festive",
    description:
      "Clean ivory body with a statement border. Designed to look expensive without being loud.",
    image: categoryHero,
    hoverImage: categoryHero,
    imageAlt: "Saree product image",
  },
  {
    id: "p2",
    name: "Violet Handpaint Floral",
    collection: "Studio",
    price: 5899,
    rating: 4.6,
    reviews: 184,
    fabric: "Silk",
    occasion: "Cocktail",
    description:
      "A modern floral story—editorial colors, fluid drape, and crisp finishing.",
    image: categoryHero,
    hoverImage: heroTwo,
    imageAlt: "Saree product image",
  },
  {
    id: "p3",
    name: "Navy Minimal with Green Edge",
    collection: "Everyday Luxury",
    price: 4999,
    rating: 4.5,
    reviews: 221,
    fabric: "Cotton Silk",
    occasion: "Office",
    description:
      "Minimal motifs with a sharp edge. Built for repeat-wear and clean styling.",
    image: heroOne,
    hoverImage: categoryHero,
    imageAlt: "Saree product image",
  },
  {
    id: "p4",
    name: "Wedding Silk — Zari Whisper",
    collection: "Occasion",
    price: 12999,
    rating: 4.8,
    reviews: 97,
    fabric: "Banarasi Silk",
    occasion: "Wedding",
    description:
      "Dense craft, refined finish. Premium without the chaos.",
    image: heroTwo,
    hoverImage: heroOne,
    imageAlt: "Saree product image",
  },
];
