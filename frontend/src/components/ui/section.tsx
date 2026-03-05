"use client";

import { cn } from "@/lib/utils";

export interface SectionProps {
  children: React.ReactNode;
  className?: string;
  /** Use for full-bleed sections that don't need horizontal container (e.g. editorial strip) */
  fullWidth?: boolean;
  id?: string;
}

/**
 * Standardizes vertical section rhythm for editorial layout.
 * Desktop: py-20 to py-28, Mobile: py-14 to py-18
 */
export function Section({
  children,
  className,
  fullWidth,
  id,
}: SectionProps) {
  return (
    <section
      id={id}
      className={cn(
        "py-14 sm:py-16 md:py-24 lg:py-28",
        !fullWidth && "mx-auto max-w-7xl px-4",
        className
      )}
    >
      {children}
    </section>
  );
}
