import React from "react";
import { Search, ShoppingBag, Heart, ChevronRight, Menu } from "lucide-react";
import AuthButtons from "../AuthButtons";

export default function Header({
  reduceMotion,
  query,
  setQuery,
  cartCount,
  wishCount,
  setMenuOpen,
  setCartOpen,
  setWishOpen,
  goTo,
  theme,
  authEnabled,
}) {
  return (
    <header
      className="sticky top-0 z-30 backdrop-blur"
      style={{ background: "rgba(247,245,240,0.8)", borderBottom: `1px solid ${theme.line}` }}
    >
      <div className="mx-auto grid max-w-7xl grid-cols-3 items-center px-4 py-3">
        <div className="flex items-center gap-2">
          <button
            onClick={() => setMenuOpen(true)}
            className="grid h-11 w-11 place-items-center rounded-full border bg-[var(--ivory)] hover:bg-white"
            style={{ "--ivory": theme.ivory, borderColor: theme.line }}
            aria-label="Open menu"
          >
            <Menu className="h-5 w-5" />
          </button>
          <button
            onClick={() => goTo("shop", !!reduceMotion)}
            className="hidden md:inline-flex items-center gap-2 text-xs font-semibold tracking-[0.18em]"
          >
            SHOP
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>

        <button
          onClick={() => goTo("top", !!reduceMotion)}
          className="mx-auto flex items-center justify-center"
          aria-label="Go to top"
        >
          <div className="flex flex-col items-center">
            <div className="text-sm font-semibold tracking-[0.35em]">SUDATTA'S</div>
            <div className="text-[10px] tracking-[0.22em] text-[#6B7280]">DESIGNER BOUTIQUE</div>
          </div>
        </button>

        <div className="ml-auto flex items-center justify-end gap-2">
          {authEnabled && (
            <div className="hidden items-center sm:flex">
              <AuthButtons />
            </div>
          )}
          <div className="hidden md:flex items-center">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[#6B7280]" />
              <input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search sarees, fabric, occasion"
                className="w-[320px] rounded-full border bg-white/60 py-2.5 pl-10 pr-4 text-sm outline-none focus:bg-white"
                style={{ borderColor: theme.line }}
              />
            </div>
          </div>

          <button
            onClick={() => setWishOpen(true)}
            className="relative grid h-11 w-11 place-items-center rounded-full border bg-[var(--ivory)] hover:bg-white"
            style={{ "--ivory": theme.ivory, borderColor: theme.line }}
            aria-label="Wishlist"
          >
            <Heart className="h-5 w-5" />
            {wishCount > 0 ? (
              <span
                className="absolute -right-1 -top-1 grid h-6 w-6 place-items-center rounded-full text-xs font-semibold text-white"
                style={{ background: theme.ink }}
              >
                {wishCount}
              </span>
            ) : null}
          </button>

          <button
            onClick={() => setCartOpen(true)}
            className="relative grid h-11 w-11 place-items-center rounded-full border bg-[var(--ivory)] hover:bg-white"
            style={{ "--ivory": theme.ivory, borderColor: theme.line }}
            aria-label="Bag"
          >
            <ShoppingBag className="h-5 w-5" />
            {cartCount > 0 ? (
              <span
                className="absolute -right-1 -top-1 grid h-6 w-6 place-items-center rounded-full text-xs font-semibold text-white"
                style={{ background: theme.ink }}
              >
                {cartCount}
              </span>
            ) : null}
          </button>
        </div>
      </div>

      <div className="md:hidden border-t" style={{ borderColor: theme.line }}>
        <div className="mx-auto max-w-7xl px-4 py-3">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[#6B7280]" />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search sarees, fabric, occasion"
              className="w-full rounded-full border bg-white/60 py-3 pl-10 pr-4 text-sm outline-none focus:bg-white"
              style={{ borderColor: theme.line }}
            />
          </div>
        </div>
      </div>
    </header>
  );
}

