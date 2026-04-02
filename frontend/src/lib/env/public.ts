import { z } from "zod";

const publicEnvSchema = z.object({
  NEXT_PUBLIC_GRAPHQL_URL: z.string().url().default("http://localhost:8080/v2"),
  NEXT_PUBLIC_SITE_URL: z.string().url().optional(),
  NEXT_PUBLIC_STORE_URL: z.string().optional(),
  NEXT_PUBLIC_IMAGE_HOST: z.string().optional(),
  NEXT_PUBLIC_DEFAULT_SHIPPING_ADDRESS_ID: z
    .string()
    .regex(/^\d+$/)
    .optional(),
  NEXT_PUBLIC_PHONE_OTP_CHANNEL: z.enum(["sms", "whatsapp"]).optional(),
});

const parsed = publicEnvSchema.safeParse({
  NEXT_PUBLIC_GRAPHQL_URL: process.env.NEXT_PUBLIC_GRAPHQL_URL,
  NEXT_PUBLIC_SITE_URL: process.env.NEXT_PUBLIC_SITE_URL,
  NEXT_PUBLIC_STORE_URL: process.env.NEXT_PUBLIC_STORE_URL,
  NEXT_PUBLIC_IMAGE_HOST: process.env.NEXT_PUBLIC_IMAGE_HOST,
  NEXT_PUBLIC_DEFAULT_SHIPPING_ADDRESS_ID:
    process.env.NEXT_PUBLIC_DEFAULT_SHIPPING_ADDRESS_ID,
  NEXT_PUBLIC_PHONE_OTP_CHANNEL: process.env.NEXT_PUBLIC_PHONE_OTP_CHANNEL,
});

if (!parsed.success) {
  const issues = parsed.error.issues
    .map((issue) => `${issue.path.join(".")}: ${issue.message}`)
    .join("; ");
  throw new Error(`Invalid public environment configuration: ${issues}`);
}

export const publicEnv = parsed.data;

export function publicGraphqlUrl(): string {
  return publicEnv.NEXT_PUBLIC_GRAPHQL_URL;
}

export function storefrontSiteOrigin(): string | null {
  const raw = publicEnv.NEXT_PUBLIC_SITE_URL;
  if (!raw) return null;
  try {
    return new URL(raw).origin;
  } catch {
    return null;
  }
}

export function configuredStorefrontOrigin(): string | null {
  const raw =
    process.env.STOREFRONT_ORIGIN ||
    publicEnv.NEXT_PUBLIC_SITE_URL ||
    (process.env.VERCEL_URL ? `https://${process.env.VERCEL_URL}` : undefined);
  if (!raw) return null;
  try {
    return new URL(raw).origin;
  } catch {
    return null;
  }
}

