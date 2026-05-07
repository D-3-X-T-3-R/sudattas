import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import { getToken } from "next-auth/jwt";

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
  matcher: ["/imtheboss/:path*"],
};
