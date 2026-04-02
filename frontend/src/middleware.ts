import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import { getToken } from "next-auth/jwt";

function adminAllowlist(): string[] {
  const raw = process.env.ADMIN_ALLOWED_EMAILS ?? "";
  return raw
    .split(",")
    .map((email) => email.trim().toLowerCase())
    .filter(Boolean);
}

function isLoginPath(pathname: string): boolean {
  return pathname === "/imtheboss/login";
}

function isProtectedAdminPath(pathname: string): boolean {
  return pathname.startsWith("/imtheboss") && !isLoginPath(pathname);
}

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  if (!isProtectedAdminPath(pathname)) {
    return NextResponse.next();
  }

  const token = await getToken({
    req: request,
    secret: process.env.AUTH_SECRET,
  });

  const email = typeof token?.email === "string" ? token.email.toLowerCase() : "";
  const allowlist = adminAllowlist();
  const authorized =
    allowlist.length > 0 && email.length > 0 && allowlist.includes(email);

  if (authorized) {
    return NextResponse.next();
  }

  const loginUrl = new URL("/imtheboss/login", request.url);
  loginUrl.searchParams.set("error", "AccessDenied");
  return NextResponse.redirect(loginUrl);
}

export const config = {
  matcher: ["/imtheboss/:path*"],
};
