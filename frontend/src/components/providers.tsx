"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { SessionProvider } from "next-auth/react";
import { useEffect, useState } from "react";
import { usePathname } from "next/navigation";
import { ensureGuestSession } from "@/lib/session";
import { ToastProvider } from "@/components/ui/toast";
import { LiveAnnouncerProvider } from "@/components/ui/live-announcer";
import { StorefrontLoginProvider } from "@/context/storefront-login-context";
import { StorefrontAuthSync } from "@/components/storefront-auth-sync";

function isAdminRoute(pathname: string | null): boolean {
  if (!pathname) return false;
  return pathname === "/imtheboss" || pathname.startsWith("/imtheboss/");
}

export function Providers({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const adminRoute = isAdminRoute(pathname);

  useEffect(() => {
    if (adminRoute) return;
    ensureGuestSession();
  }, [adminRoute]);

  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 60 * 1000,
          },
        },
      })
  );
  return (
    <SessionProvider>
      {adminRoute ? null : <StorefrontAuthSync />}
      <QueryClientProvider client={queryClient}>
        <LiveAnnouncerProvider>
          {adminRoute ? (
            <ToastProvider>{children}</ToastProvider>
          ) : (
            <StorefrontLoginProvider>
              <ToastProvider>{children}</ToastProvider>
            </StorefrontLoginProvider>
          )}
        </LiveAnnouncerProvider>
      </QueryClientProvider>
    </SessionProvider>
  );
}
