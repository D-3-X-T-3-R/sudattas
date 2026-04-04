import "server-only";

import { z } from "zod";
import { publicEnv, storefrontSiteOrigin } from "@/lib/env/public";

const serverEnvSchema = z.object({
  GRAPHQL_URL: z.string().url().optional(),
  GRAPHQL_METRICS_URL: z.string().url().optional(),
  CORE_OPS_METRICS_URL: z.string().url().optional(),
  STOREFRONT_ORIGIN: z.string().url().optional(),
  AUTH_SECRET: z.string().min(1).optional(),
  NEXTAUTH_URL: z.string().url().optional(),
  GOOGLE_CLIENT_ID: z.string().min(1).optional(),
  GOOGLE_CLIENT_SECRET: z.string().min(1).optional(),
  ADMIN_ALLOWED_EMAILS: z.string().optional(),
  INTERNAL_API_SECRET: z.string().min(1).optional(),
});

const parsed = serverEnvSchema.safeParse({
  GRAPHQL_URL: process.env.GRAPHQL_URL,
  GRAPHQL_METRICS_URL: process.env.GRAPHQL_METRICS_URL,
  CORE_OPS_METRICS_URL: process.env.CORE_OPS_METRICS_URL,
  STOREFRONT_ORIGIN: process.env.STOREFRONT_ORIGIN,
  AUTH_SECRET: process.env.AUTH_SECRET,
  NEXTAUTH_URL: process.env.NEXTAUTH_URL,
  GOOGLE_CLIENT_ID: process.env.GOOGLE_CLIENT_ID,
  GOOGLE_CLIENT_SECRET: process.env.GOOGLE_CLIENT_SECRET,
  ADMIN_ALLOWED_EMAILS: process.env.ADMIN_ALLOWED_EMAILS,
  INTERNAL_API_SECRET: process.env.INTERNAL_API_SECRET,
});

if (!parsed.success) {
  const issues = parsed.error.issues
    .map((issue) => `${issue.path.join(".")}: ${issue.message}`)
    .join("; ");
  throw new Error(`Invalid server environment configuration: ${issues}`);
}

export const serverEnv = parsed.data;

export function serverGraphqlUrl(): string {
  return serverEnv.GRAPHQL_URL ?? publicEnv.NEXT_PUBLIC_GRAPHQL_URL;
}

export function graphqlBaseUrl(): string {
  return serverGraphqlUrl().replace(/\/v2\/?$/, "");
}

export function graphqlMetricsUrl(): string {
  return serverEnv.GRAPHQL_METRICS_URL ?? `${graphqlBaseUrl()}/metrics`;
}

export function coreOpsMetricsUrl(): string {
  return serverEnv.CORE_OPS_METRICS_URL ?? "http://127.0.0.1:9090/metrics";
}

export function serverStorefrontOrigin(): string | null {
  if (serverEnv.STOREFRONT_ORIGIN) {
    try {
      return new URL(serverEnv.STOREFRONT_ORIGIN).origin;
    } catch {
      return null;
    }
  }
  return storefrontSiteOrigin();
}

