import { z } from "zod";

export const productSchema = z.object({
  id: z.string(),
  name: z.string(),
  collection: z.string(),
  price: z.number(),
  rating: z.number(),
  reviews: z.number().optional(),
  fabric: z.string(),
  occasion: z.string(),
  description: z.string(),
  image: z.string(),
  hoverImage: z.string().optional(),
  imageAlt: z.string().optional(),
});

export const collectionSchema = z.object({
  key: z.string(),
  blurb: z.string(),
});

export const cartLineSchema = z.object({
  product: productSchema,
  qty: z.number().int().min(1),
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
  priceRupees: z.string(),
  stockQuantity: z.string(),
  categoryId: z.string().min(1, "Please select a category"),
});
export const adminNewCategorySchema = z.object({ name: z.string().min(1) });
export type AdminProductForm = z.infer<typeof adminProductFormSchema>;
export type AdminNewCategory = z.infer<typeof adminNewCategorySchema>;
