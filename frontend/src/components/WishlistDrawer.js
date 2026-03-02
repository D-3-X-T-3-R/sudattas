import React from "react";
import Drawer from "./Drawer";

export default function WishlistDrawer({
  open,
  onClose,
  wishCount,
  wishedProducts,
  onQuickView,
  onAddToCart,
  onToggleWish,
  theme,
  INR,
}) {
  return (
    <Drawer
      open={open}
      title={`WISHLIST (${wishCount})`}
      onClose={onClose}
      side="right"
      theme={theme}
    >
      {wishedProducts.length === 0 ? (
        <div className="rounded-2xl bg-white p-6 text-sm text-[#374151]">
          No items yet.
        </div>
      ) : (
        <div className="space-y-4">
          {wishedProducts.map((p) => (
            <div
              key={p.id}
              className="border-b pb-4"
              style={{ borderColor: theme.line }}
            >
              <div className="text-[11px] tracking-[0.18em] text-[#6B7280]">
                {p.collection.toUpperCase()}
              </div>
              <div className="mt-1 text-sm font-semibold text-[#111]">
                {p.name}
              </div>
              <div className="mt-2 flex items-center justify-between">
                <div className="text-sm font-semibold">
                  {INR.format(p.price)}
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => onQuickView(p)}
                    className="rounded-full border bg-white px-4 py-2 text-xs font-semibold hover:bg-white/80"
                    style={{ borderColor: theme.line }}
                  >
                    Quick view
                  </button>
                  <button
                    onClick={() => onAddToCart(p)}
                    className="rounded-full px-4 py-2 text-xs font-semibold text-white"
                    style={{ background: theme.ink }}
                  >
                    Add
                  </button>
                </div>
              </div>
              <button
                onClick={() => onToggleWish(p)}
                className="mt-3 text-xs font-semibold tracking-[0.18em] text-[#6B7280] hover:text-[#111]"
              >
                REMOVE
              </button>
            </div>
          ))}
        </div>
      )}
    </Drawer>
  );
}

