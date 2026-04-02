"use client";

import { createContext, useContext } from "react";
import { signIn } from "next-auth/react";

type StorefrontLoginContextValue = {
  openLogin: (callbackUrl?: string) => void;
};

const StorefrontLoginContext = createContext<StorefrontLoginContextValue>({
  openLogin: (_callbackUrl?: string) => undefined,
});

export function StorefrontLoginProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const openLogin = (callbackUrl?: string) => {
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
