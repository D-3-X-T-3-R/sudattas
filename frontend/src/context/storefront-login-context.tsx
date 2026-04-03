"use client";

import { createContext, useContext } from "react";
import { signIn } from "next-auth/react";
import { trackClientTelemetry } from "@/lib/client-telemetry";

type StorefrontLoginContextValue = {
  openLogin: (callbackUrl?: string) => void;
};

const StorefrontLoginContext = createContext<StorefrontLoginContextValue>({
  openLogin: () => undefined,
});

export function StorefrontLoginProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const openLogin = (callbackUrl?: string) => {
    trackClientTelemetry({
      route:
        callbackUrl ||
        (typeof window !== "undefined" ? window.location.pathname : "/"),
      userMode: "public",
      action: "AUTH_SIGN_IN_GOOGLE_ATTEMPT",
      outcome: "success",
      errorClass: "none",
      errorCode: null,
      message: "Storefront sign-in attempt initiated.",
      status: 200,
    });
    void signIn("google", callbackUrl ? { callbackUrl } : undefined);
  };

  return (
    <StorefrontLoginContext.Provider value={{ openLogin }}>
      {children}
    </StorefrontLoginContext.Provider>
  );
}

export function useStorefrontLogin(): StorefrontLoginContextValue {
  return useContext(StorefrontLoginContext);
}
