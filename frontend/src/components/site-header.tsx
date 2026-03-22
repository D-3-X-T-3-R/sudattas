"use client";

import { useState } from "react";
import { useReducedMotion } from "framer-motion";
import { Header } from "@/components/header";
import { MenuDrawer } from "@/components/menu-drawer";
import { useStorefront } from "@/context/storefront-context";
import { useLockBodyScroll } from "@/hooks/use-lock-body-scroll";

/**
 * Same navbar as the landing page (font, colors, search toggle, menu drawer, icons).
 * Use on routes outside `/` where nav links go to `/#section` hashes.
 */
export function SiteHeader() {
  const { cartCount, wishCount } = useStorefront();
  const [query, setQuery] = useState("");
  const [menuOpen, setMenuOpen] = useState(false);
  const reduceMotion = useReducedMotion();
  useLockBodyScroll(menuOpen);

  return (
    <>
      <Header
        query={query}
        setQuery={setQuery}
        cartCount={cartCount}
        wishCount={wishCount}
        setMenuOpen={setMenuOpen}
        setCartOpen={() => {}}
        goTo={() => {}}
        navUseHashLinks
      />
      <MenuDrawer
        open={menuOpen}
        onClose={() => setMenuOpen(false)}
        setCollection={() => {}}
        reduceMotion={!!reduceMotion}
      />
    </>
  );
}
