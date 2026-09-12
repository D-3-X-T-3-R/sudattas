import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import { getToken } from "next-auth/jwt";
import {
  STOREFRONT_COMING_SOON_PATH,
  STOREFRONT_GATE_HEADER,
  STOREFRONT_GATE_REASON_HEADER,
  STOREFRONT_GATE_REASON_PARAM,
  type StorefrontGateReason,
  isApiPath,
  isComingSoonPath,
  isNextInternalPath,
  isPublicStorefrontPath,
  isStaticAssetPath,
  isStorefrontReady,
  isStorefrontBackendAvailable,
  shouldGateStorefrontPath,
} from "@/lib/storefront-readiness";

export const ACCOUNT_DEACTIVATED_PATH = "/account-deactivated";

function isLoginPath(pathname: string): boolean {
  return pathname === "/imtheboss/login";
}

function isProtectedAdminPath(pathname: string): boolean {
  return pathname.startsWith("/imtheboss") && !isLoginPath(pathname);
}

function isAccountDeactivatedPath(pathname: string): boolean {
  return pathname === ACCOUNT_DEACTIVATED_PATH;
}

/// Paths where checking the session token is pointless overhead — mirrors the middleware
/// `matcher` config below, which already excludes most of these from ever reaching this
/// function in production; kept here too since unit tests call `middleware()` directly.
function skipsTokenCheck(pathname: string): boolean {
  return isApiPath(pathname) || isNextInternalPath(pathname) || isStaticAssetPath(pathname);
}

// How long a "not deactivated" / "deactivated" answer from the backend is trusted before the
// next request re-checks live. Deliberately short and NOT tied to the customer's own session
// cookie: this is checked fresh via internal-service auth (X-Internal-Auth + the customer's
// stable numeric ID), independent of the customer's own Google idToken. That sidesteps the bug
// an earlier version of this had — caching the flag *inside* the NextAuth JWT and re-probing it
// only when a separate client-side session round-trip happened to fire meant a fresh
// deactivation (or reactivation) could take up to that interval to ever be re-checked at all,
// since middleware's `getToken()` only reads whatever is already baked into the cookie and never
// runs the probe itself. This cache exists purely to avoid duplicate backend calls on rapid
// navigation, not as the source of truth for how fresh the check is allowed to be.
const ACCOUNT_STATUS_CACHE_TTL_MS = 10_000;
const accountStatusCache = new Map<string, { deactivated: boolean; expiresAt: number }>();

function graphqlBaseUrlForMiddleware(): string {
  return process.env.GRAPHQL_URL ?? process.env.NEXT_PUBLIC_GRAPHQL_URL ?? "http://localhost:8080/v2";
}

/// Live check via internal-service auth (never the customer's own Google idToken, which has its
/// own separate, much longer-lived expiry/refresh cycle this check has no business depending on).
/// Fails open (not deactivated) on any missing config, network error, or backend error — this
/// only drives a UX redirect; real enforcement is the backend's own `jwt_user_id()` gate on every
/// actual mutation/query, regardless of what this returns.
async function isAccountDeactivatedLive(customerUserId: string): Promise<boolean> {
  const now = Date.now();
  const cached = accountStatusCache.get(customerUserId);
  if (cached && cached.expiresAt > now) {
    return cached.deactivated;
  }

  const secret = process.env.INTERNAL_API_SECRET?.trim();
  if (!secret) {
    return false;
  }

  try {
    const res = await fetch(graphqlBaseUrlForMiddleware(), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Internal-Auth": secret,
        "X-Customer-User-Id": customerUserId,
      },
      body: JSON.stringify({
        query: "query MiddlewareAccountStatus { authInfo { accountDeactivated } }",
      }),
      cache: "no-store",
    });
    const json = (await res.json().catch(() => ({}))) as {
      data?: { authInfo?: { accountDeactivated?: boolean } };
      errors?: Array<{ message?: string }>;
    };
    if (!res.ok || json.errors?.length) {
      return false;
    }
    const deactivated = json.data?.authInfo?.accountDeactivated === true;
    accountStatusCache.set(customerUserId, {
      deactivated,
      expiresAt: now + ACCOUNT_STATUS_CACHE_TTL_MS,
    });
    return deactivated;
  } catch {
    return false;
  }
}

function gatedRequestHeaders(
  request: NextRequest,
  reason: StorefrontGateReason
): Headers {
  const headers = new Headers(request.headers);
  headers.set(STOREFRONT_GATE_HEADER, "1");
  headers.set(STOREFRONT_GATE_REASON_HEADER, reason);
  return headers;
}

function comingSoonUrl(request: NextRequest, reason: StorefrontGateReason): URL {
  const url = new URL(STOREFRONT_COMING_SOON_PATH, request.url);
  url.searchParams.set(STOREFRONT_GATE_REASON_PARAM, reason);
  return url;
}

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const storefrontReady = isStorefrontReady();

  if (isComingSoonPath(pathname)) {
    const reason: StorefrontGateReason | null = storefrontReady
      ? (await isStorefrontBackendAvailable())
        ? null
        : "service-unavailable"
      : "not-ready";

    if (!reason) {
      return NextResponse.redirect(new URL("/", request.url));
    }

    return NextResponse.next({
      request: {
        headers: gatedRequestHeaders(request, reason),
      },
    });
  }

  if (isPublicStorefrontPath(pathname)) {
    if (shouldGateStorefrontPath(pathname, storefrontReady)) {
      return NextResponse.rewrite(comingSoonUrl(request, "not-ready"), {
        request: {
          headers: gatedRequestHeaders(request, "not-ready"),
        },
      });
    }

    if (!(await isStorefrontBackendAvailable())) {
      return NextResponse.rewrite(comingSoonUrl(request, "service-unavailable"), {
        request: {
          headers: gatedRequestHeaders(request, "service-unavailable"),
        },
      });
    }
  }

  if (!isProtectedAdminPath(pathname)) {
    if (isLoginPath(pathname) || isAccountDeactivatedPath(pathname) || skipsTokenCheck(pathname)) {
      return NextResponse.next();
    }

    // A deactivated/suspended customer keeps a valid NextAuth session (sign-in itself isn't
    // gated — see auth.ts) but every real backend call they make fails at the GraphQL auth
    // gate. Rather than let them wander a storefront that errors out everywhere, send them
    // straight to the one page that explains why and how to get help.
    const token = await getToken({
      req: request,
      secret: process.env.AUTH_SECRET ?? process.env.NEXTAUTH_SECRET,
    });
    const customerUserId = token && (token as { customerUserId?: string }).customerUserId;
    if (customerUserId && (await isAccountDeactivatedLive(customerUserId))) {
      return NextResponse.redirect(new URL(ACCOUNT_DEACTIVATED_PATH, request.url));
    }
    return NextResponse.next();
  }

  const token = await getToken({
    req: request,
    secret: process.env.AUTH_SECRET ?? process.env.NEXTAUTH_SECRET,
  });

  if (token && (token as { isAdmin?: boolean }).isAdmin === true) {
    return NextResponse.next();
  }

  const loginUrl = new URL("/imtheboss/login", request.url);
  loginUrl.searchParams.set("error", "AccessDenied");
  return NextResponse.redirect(loginUrl);
}

export const config = {
  matcher: ["/((?!api|_next/static|_next/image|favicon.ico|.*\\..*).*)"],
};
