"use client";

import { cn } from "@/lib/utils";

export interface SectionProps {
  children: React.ReactNode;
  className?: string;
  /** Use for full-bleed sections that don't need horizontal container (e.g. editorial strip) */
  fullWidth?: boolean;
  /** Reduced vertical padding (e.g. for admin panels) */
  compact?: boolean;
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
  id,
}: SectionProps) {
  return (
    <section
      id={id}
      className={cn(
        compact ? "pt-8 md:pt-10" : "pt-16 md:pt-20",
        !fullWidth && "mx-auto max-w-[2000px] px-4",
        className
      )}
    >
      {children}
    </section>
  );
}
