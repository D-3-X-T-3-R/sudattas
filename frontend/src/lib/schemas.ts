import { z } from "zod";
import { rupeesInputSchema } from "@/lib/validation-schemas";

export const productSchema = z.object({
  id: z.string(),
  name: z.string(),
  collection: z.string(),
  price: z.number(),
  pricePaise: z.number().int().nonnegative().optional(),
  /** Backend display string (e.g. "₹799.99"); use for display when set to avoid rounding. */
  priceFormatted: z.string().optional(),
  rating: z.number(),
  reviews: z.number().optional(),
  fabric: z.string(),
  occasion: z.string(),
  description: z.string(),
  image: z.string(),
  hoverImage: z.string().optional(),
  /** All image URLs for gallery; when set, use instead of just image + hoverImage. */
  images: z.array(z.string()).optional(),
  imageAlt: z.string().optional(),
  /** Per-size stock from DB. Empty or absent = free size (no size selector). */
  variantStock: z
    .array(
      z.object({
        variantId: z.string().optional(),
        sizeId: z.string(),
        sizeName: z.string(),
        quantity: z.number(),
      })
    )
    .optional(),
});

export const collectionSchema = z.object({
  key: z.string(),
  blurb: z.string(),
});

export const cartLineSchema = z.object({
  id: z.string(),
  product: productSchema,
  qty: z.number().int().min(1),
  sizeName: z.string().optional().nullable(),
});

export const paymentIntentSchema = z.object({
  intentId: z.string().optional(),
  razorpayOrderId: z.string(),
  razorpayKeyId: z.string(),
  orderId: z.string(),
  amountPaise: z.string(),
  currency: z.string(),
});

export const verifyRazorpayPayloadSchema = z.object({
  verified: z.boolean(),
  paymentIntent: z
    .object({
      status: z.string(),
    })
    .optional(),
});

export type Product = z.infer<typeof productSchema>;
export type Collection = z.infer<typeof collectionSchema>;
export type CartLine = z.infer<typeof cartLineSchema>;
export type PaymentIntent = z.infer<typeof paymentIntentSchema>;
export type VerifyRazorpayPayload = z.infer<typeof verifyRazorpayPayloadSchema>;

/* Admin: form and API shapes */
export const adminProductFormSchema = z.object({
  name: z.string().min(1, "Name is required"),
  description: z.string(),
  priceRupees: rupeesInputSchema,
  categoryId: z.string().min(1, "Please select a category"),
  sku: z
    .string()
    .trim()
    .regex(/^[A-Za-z0-9_-]{1,128}$/, "SKU may only contain letters, numbers, hyphen and underscore")
    .optional()
    .or(z.literal("")),
  slug: z
    .string()
    .trim()
    .regex(/^[A-Za-z0-9_-]{1,128}$/, "Slug may only contain letters, numbers, hyphen and underscore")
    .optional()
    .or(z.literal("")),
  fabric: z.string().optional(),
  weave: z.string().optional(),
  occasion: z.string().optional(),
  hasBlousePiece: z.boolean().optional(),
  careInstructions: z.string().optional(),
  productStatusId: z.string().optional(),
});
export const adminNewCategorySchema = z.object({ name: z.string().min(1) });
export type AdminProductForm = z.infer<typeof adminProductFormSchema>;
export type AdminNewCategory = z.infer<typeof adminNewCategorySchema>;

/** One variant row for add-product form (size, color, extra price, initial stock) */
export interface AdminProductVariantRow {
  variantId?: string;
  sizeId: string;
  colorId?: string;
  additionalPricePaise: string;
  quantityAvailable: string;
  reorderLevel?: string;
}

/** Attribute to link to product (existing attributeId or new name+value) */
export interface AdminProductAttributeRow {
  attributeId?: string;
  attributeName: string;
  attributeValue: string;
}
