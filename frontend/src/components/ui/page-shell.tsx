"use client";

import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

type PageShellProps = {
  children: ReactNode;
  className?: string;
  containerClassName?: string;
  wide?: boolean;
};

export function PageShell({
  children,
  className,
  containerClassName,
  wide = false,
}: PageShellProps) {
  return (
    <div className={cn("min-h-screen bg-[var(--background)] text-[var(--foreground)]", className)}>
      <main
        className={cn(
          "mx-auto w-full px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]",
          wide ? "max-w-[1440px]" : "max-w-[var(--container-max)]",
          containerClassName
        )}
      >
        {children}
      </main>
    </div>
  );
}

type SectionHeaderProps = {
  label?: string;
  title: string;
  description?: string;
  action?: ReactNode;
  className?: string;
};

export function SectionHeader({
  label,
  title,
  description,
  action,
  className,
}: SectionHeaderProps) {
  return (
    <header
      className={cn(
        "flex flex-col gap-4 border-b border-[var(--color-line)] pb-6 md:flex-row md:items-end md:justify-between",
        className
      )}
    >
      <div className="max-w-2xl">
        {label ? (
          <p className="text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
            {label}
          </p>
        ) : null}
        <h2 className="mt-1 font-display text-[1.95rem] leading-[1.2] tracking-[-0.01em] text-[var(--color-ink)] md:text-[2.4rem]">
          {title}
        </h2>
        {description ? (
          <p className="mt-2 text-base text-[var(--color-muted)]">{description}</p>
        ) : null}
      </div>
      {action ? <div className="shrink-0">{action}</div> : null}
    </header>
  );
}

type SummaryCardProps = {
  title: string;
  subtitle?: string;
  children: ReactNode;
  className?: string;
};

export function SummaryCard({ title, subtitle, children, className }: SummaryCardProps) {
  return (
    <section className={cn("rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)] md:p-5", className)}>
      <div className="border-b border-[var(--color-line)] pb-3">
        <p className="text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">{title}</p>
        {subtitle ? <p className="mt-1 text-sm text-[var(--color-muted)]">{subtitle}</p> : null}
      </div>
      <div className="pt-4">{children}</div>
    </section>
  );
}

type EmptyStateProps = {
  title: string;
  description: string;
  action?: ReactNode;
  className?: string;
};

export function EmptyState({ title, description, action, className }: EmptyStateProps) {
  return (
    <section className={cn("rounded-lg border border-dashed border-[var(--color-line-strong)] bg-[var(--color-surface-soft)] p-8 text-center", className)}>
      <h3 className="font-display text-2xl text-[var(--color-ink)]">{title}</h3>
      <p className="mt-2 text-sm text-[var(--color-muted)]">{description}</p>
      {action ? <div className="mt-5">{action}</div> : null}
    </section>
  );
}

type ErrorStateProps = {
  title: string;
  message: string;
  action?: ReactNode;
  className?: string;
};

export function ErrorState({ title, message, action, className }: ErrorStateProps) {
  return (
    <section className={cn("rounded-lg border border-[#D7B6A8] bg-[#FFF5EF] p-6", className)}>
      <p className="text-[11px] font-semibold uppercase tracking-[0.2em] text-[#9F4A35]">Issue</p>
      <h3 className="mt-2 font-display text-2xl text-[var(--color-ink)]">{title}</h3>
      <p className="mt-2 text-sm text-[#7A5348]">{message}</p>
      {action ? <div className="mt-4">{action}</div> : null}
    </section>
  );
}
