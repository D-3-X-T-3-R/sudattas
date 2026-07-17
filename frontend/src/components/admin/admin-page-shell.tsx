"use client";

import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

export function AdminPageShell({
  label,
  title,
  description,
  action,
  children,
  className,
}: {
  label?: string;
  title: string;
  description?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={cn("mx-auto w-full max-w-[1280px] space-y-6", className)}>
      <header className="rounded-2xl border border-[var(--color-line)] bg-[var(--admin-surface-muted)] px-5 py-6 shadow-[var(--admin-card-shadow)] md:px-7 md:py-7">
        <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
          <div>
            {label ? (
              <p className="text-sm font-medium text-[var(--color-muted)]">{label}</p>
            ) : null}
            <h1 className="mt-1 font-display text-[2.1rem] leading-[1.2] text-[var(--color-ink)] md:text-[2.4rem]">{title}</h1>
            {description ? <p className="mt-2.5 text-base leading-relaxed text-[var(--color-muted)]">{description}</p> : null}
          </div>
          {action ? <div className="shrink-0">{action}</div> : null}
        </div>
      </header>
      {children}
    </section>
  );
}
