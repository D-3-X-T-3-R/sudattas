"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { SessionProvider } from "next-auth/react";
import { useEffect, useState } from "react";
import { ensureGuestSession } from "@/lib/session";
import { ToastProvider } from "@/components/ui/toast";
import { LiveAnnouncerProvider } from "@/components/ui/live-announcer";
import { StorefrontLoginProvider } from "@/context/storefront-login-context";

export function Providers({ children }: { children: React.ReactNode }) {
  useEffect(() => {
    ensureGuestSession();
  }, []);

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
      <QueryClientProvider client={queryClient}>
        <StorefrontLoginProvider>
          <LiveAnnouncerProvider>
            <ToastProvider>{children}</ToastProvider>
          </LiveAnnouncerProvider>
        </StorefrontLoginProvider>
      </QueryClientProvider>
    </SessionProvider>
  );
}
