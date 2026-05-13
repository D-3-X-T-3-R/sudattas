import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import { getToken } from "next-auth/jwt";
import {
  STOREFRONT_COMING_SOON_PATH,
  STOREFRONT_GATE_HEADER,
  STOREFRONT_GATE_REASON_HEADER,
  STOREFRONT_GATE_REASON_PARAM,
  type StorefrontGateReason,
  isComingSoonPath,
  isPublicStorefrontPath,
  isStorefrontReady,
  isStorefrontBackendAvailable,
  shouldGateStorefrontPath,
} from "@/lib/storefront-readiness";

function isLoginPath(pathname: string): boolean {
  return pathname === "/imtheboss/login";
}

function isProtectedAdminPath(pathname: string): boolean {
  return pathname.startsWith("/imtheboss") && !isLoginPath(pathname);
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
