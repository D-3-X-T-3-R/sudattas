"use client";

import { useActiveSection } from "@/hooks/use-active-section";
import { goTo } from "@/hooks/use-scroll-to";

const SECTIONS = [
  { id: "top", label: "Home" },
  { id: "collections", label: "Collections" },
  { id: "shop", label: "Shop" },
  { id: "story", label: "Story" },
] as const;

export function SectionNav() {
  const active = useActiveSection(SECTIONS.map((s) => s.id));

  return (
    <nav
      className="fixed right-4 top-1/2 z-20 hidden -translate-y-1/2 flex-col gap-3 lg:flex"
      aria-label="Page sections"
    >
      {SECTIONS.map((s) => (
        <button
          key={s.id}
          type="button"
          onClick={() => goTo(s.id, false)}
          className="group flex items-center justify-end gap-2"
          aria-label={`Go to ${s.label}`}
          aria-current={active === s.id ? "true" : undefined}
        >
          <span
            className="text-[10px] font-medium tracking-widest text-[var(--color-muted)] opacity-0 transition-opacity group-hover:opacity-100"
          >
            {s.label}
          </span>
          <span
            className="h-2 w-2 rounded-full border border-[var(--color-line)] transition-all duration-200"
            style={{
              background: active === s.id ? "var(--color-accent-gold)" : "transparent",
              transform: active === s.id ? "scale(1.25)" : "scale(1)",
            }}
          />
        </button>
      ))}
    </nav>
  );
}
