"use client";

import { cn } from "@/lib/utils";

export interface SectionProps {
  children: React.ReactNode;
  className?: string;
  /** Use for full-bleed sections that don't need horizontal container (e.g. editorial strip) */
  fullWidth?: boolean;
  /** Reduced vertical padding (e.g. for admin panels) */
  compact?: boolean;
  soft?: boolean;
  id?: string;
}

/**
 * Standardizes vertical section rhythm for editorial layout.
 */
export function Section({
  children,
  className,
  fullWidth,
  compact,
  soft,
  id,
}: SectionProps) {
  return (
    <section
      id={id}
      className={cn(
        compact ? "pt-8 md:pt-10" : "pt-12 md:pt-16",
        !fullWidth && "mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]",
        soft && "rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 md:p-6",
        className
      )}
    >
      {children}
    </section>
  );
}

export const StorefrontSection = Section;
