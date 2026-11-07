import { z } from "zod";

// In production, a missing NEXT_PUBLIC_GRAPHQL_URL must fail loudly at boot — silently falling
// back to the localhost dev default would otherwise point every GraphQL call at localhost in a
// real deployment, a total but hard-to-diagnose outage instead of a fail-fast startup error.
//
// Either an absolute URL, or a same-origin path like "/v2" when the reverse proxy (Caddy, see
// backend/Caddyfile) serves the backend on the site's own origin. A path is only usable from the
// browser; server-side code resolves it via GRAPHQL_URL (see publicGraphqlUrl).
const absoluteUrlOrPath = z
  .string()
  .refine((value) => value.startsWith("/") || z.string().url().safeParse(value).success, {
    message: "must be an absolute URL or a path starting with /",
  });
const graphqlUrlSchema =
  process.env.NODE_ENV === "production"
    ? absoluteUrlOrPath
    : absoluteUrlOrPath.default("http://localhost:8080/v2");

const publicEnvSchema = z.object({
  NEXT_PUBLIC_GRAPHQL_URL: graphqlUrlSchema,
  NEXT_PUBLIC_SITE_URL: z.string().url().optional(),
  NEXT_PUBLIC_STORE_URL: z.string().optional(),
  NEXT_PUBLIC_IMAGE_HOST: z.string().optional(),
  NEXT_PUBLIC_DEFAULT_SHIPPING_ADDRESS_ID: z
    .string()
    .regex(/^\d+$/)
    .optional(),
  NEXT_PUBLIC_PHONE_OTP_CHANNEL: z.enum(["sms", "whatsapp"]).optional(),
  NEXT_PUBLIC_FREE_SHIPPING_THRESHOLD_MINOR: z.coerce
    .number()
    .int()
    .nonnegative()
    .default(100000),
});

const parsed = publicEnvSchema.safeParse({
  NEXT_PUBLIC_GRAPHQL_URL: process.env.NEXT_PUBLIC_GRAPHQL_URL,
  NEXT_PUBLIC_SITE_URL: process.env.NEXT_PUBLIC_SITE_URL,
  NEXT_PUBLIC_STORE_URL: process.env.NEXT_PUBLIC_STORE_URL,
  NEXT_PUBLIC_IMAGE_HOST: process.env.NEXT_PUBLIC_IMAGE_HOST,
  NEXT_PUBLIC_DEFAULT_SHIPPING_ADDRESS_ID:
    process.env.NEXT_PUBLIC_DEFAULT_SHIPPING_ADDRESS_ID,
  NEXT_PUBLIC_PHONE_OTP_CHANNEL: process.env.NEXT_PUBLIC_PHONE_OTP_CHANNEL,
  NEXT_PUBLIC_FREE_SHIPPING_THRESHOLD_MINOR:
    process.env.NEXT_PUBLIC_FREE_SHIPPING_THRESHOLD_MINOR,
});

if (!parsed.success) {
  const issues = parsed.error.issues
    .map((issue) => `${issue.path.join(".")}: ${issue.message}`)
    .join("; ");
  throw new Error(`Invalid public environment configuration: ${issues}`);
}

export const publicEnv = parsed.data;

/// GraphQL endpoint for the current runtime. In the browser this is NEXT_PUBLIC_GRAPHQL_URL as
/// configured (possibly a same-origin path). On the server a relative path can't be fetched, so
/// it resolves to GRAPHQL_URL — the backend's direct address, bypassing the reverse proxy.
export function publicGraphqlUrl(): string {
  const url = publicEnv.NEXT_PUBLIC_GRAPHQL_URL;
  if (typeof window !== "undefined" || !url.startsWith("/")) {
    return url;
  }
  const serverUrl = process.env.GRAPHQL_URL?.trim();
  if (serverUrl) return serverUrl;
  if (process.env.NODE_ENV === "production") {
    throw new Error(
      "GRAPHQL_URL must be set when NEXT_PUBLIC_GRAPHQL_URL is a relative path (server-side calls need an absolute URL)"
    );
  }
  return `http://localhost:8080${url}`;
}

export function freeShippingThresholdMinor(): number {
  return publicEnv.NEXT_PUBLIC_FREE_SHIPPING_THRESHOLD_MINOR;
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

