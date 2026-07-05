"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useReducedMotion } from "framer-motion";
import { User } from "lucide-react";
import { useSession } from "next-auth/react";
import { Header } from "@/components/header";
import { MenuDrawer } from "@/components/menu-drawer";
import { MobileBottomBar } from "@/components/mobile-bottom-bar";
import { useStorefront } from "@/context/storefront-context";
import { useStorefrontLogin } from "@/context/storefront-login-context";
import { useLockBodyScroll } from "@/hooks/use-lock-body-scroll";
import { goTo } from "@/hooks/use-scroll-to";

/** Global storefront header. Admin routes opt out in this component. */
export function SiteHeader() {
  const pathname = usePathname() ?? "";
  const { cartCount, wishCount } = useStorefront();
  const { status, data: session } = useSession();
  const { openLogin } = useStorefrontLogin();
  const [query, setQuery] = useState("");
  const [menuOpen, setMenuOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const reduceMotion = !!useReducedMotion();
  useLockBodyScroll(menuOpen);

  useEffect(() => {
    const className = "storefront-mobile-utilities";
    if (pathname.startsWith("/imtheboss")) {
      document.documentElement.classList.remove(className);
      return;
    }

    document.documentElement.classList.add(className);
    return () => document.documentElement.classList.remove(className);
  }, [pathname]);

  const firstName = useMemo(() => {
    const rawName = session?.user?.name?.trim() ?? "";
    const looksLikePhone = /^\+?\d{10,15}$/.test(rawName);
    return !rawName || looksLikePhone ? "Profile" : rawName.split(/\s+/)[0];
  }, [session?.user?.name]);

  if (pathname.startsWith("/imtheboss")) return null;

  return (
    <>
      <Header
        query={query}
        setQuery={setQuery}
        cartCount={cartCount}
        wishCount={wishCount}
        setMenuOpen={setMenuOpen}
        goTo={(id, instant) => goTo(id, instant ?? reduceMotion)}
        searchOpen={searchOpen}
        setSearchOpen={setSearchOpen}
        navUseHashLinks={pathname !== "/"}
        authEnabled
        authButtons={
          status === "authenticated" ? (
            <Link
              href="/profile"
              className="inline-flex h-9 items-center gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)] transition-colors hover:border-[var(--color-gold)] hover:text-[var(--color-green)]"
              aria-label="Open profile"
            >
              <User size={14} />
              {firstName}
            </Link>
          ) : (
            <button
              type="button"
              onClick={() => openLogin(pathname || "/")}
              className="inline-flex h-9 items-center gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)] transition-colors hover:border-[var(--color-gold)] hover:text-[var(--color-green)]"
              aria-label="Sign in"
            >
              <User size={14} />
              Sign In
            </button>
          )
        }
      />
      <MenuDrawer
        open={menuOpen}
        onClose={() => setMenuOpen(false)}
        setCollection={() => {}}
        reduceMotion={reduceMotion}
      />
      <MobileBottomBar
        wishCount={wishCount}
        cartCount={cartCount}
        authenticated={status === "authenticated"}
        onProfileOpen={() => openLogin(pathname || "/")}
        onSearchOpen={() => setSearchOpen((open) => !open)}
      />
    </>
  );
}
