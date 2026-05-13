"use client";

import { formatInrFromPaise } from "@/lib/money";
import { cn } from "@/lib/utils";

export type StorefrontFilterOption = { value: string; label: string; count: number };

function FilterGroupShell({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="border-t border-[var(--color-line)] pt-4 first:border-t-0 first:pt-0">
      <h3 className="text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]">
        {title}
      </h3>
      {children}
    </section>
  );
}

export function SelectFilter({
  title,
  id,
  value,
  options,
  onChange,
  allLabel = "All",
}: {
  title: string;
  id: string;
  value: string;
  options: StorefrontFilterOption[];
  onChange: (value: string) => void;
  allLabel?: string;
}) {
  return (
    <FilterGroupShell title={title}>
      <label className="mt-3 block" htmlFor={id}>
        <span className="sr-only">{title}</span>
        <select
          id={id}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          className="h-10 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 text-sm text-[var(--color-ink)] outline-none focus:border-[var(--color-green)] focus:ring-2 focus:ring-[var(--color-focus)]"
        >
          <option value="">{allLabel}</option>
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>
    </FilterGroupShell>
  );
}

export function CheckboxGroup({
  title,
  options,
  selected,
  onToggle,
}: {
  title: string;
  options: StorefrontFilterOption[];
  selected: string[];
  onToggle: (value: string) => void;
}) {
  return (
    <FilterGroupShell title={title}>
      <div className="mt-3 space-y-1.5">
        {options.map((option) => (
          <label
            key={option.value}
            className="flex min-h-10 cursor-pointer items-center justify-between gap-3 rounded-md px-1.5 py-2 text-sm text-[var(--color-ink)] hover:bg-[var(--color-surface-soft)]"
          >
            <span className="flex min-w-0 items-center gap-2">
              <input
                type="checkbox"
                checked={selected.includes(option.value)}
                onChange={() => onToggle(option.value)}
                className="h-4 w-4 shrink-0 rounded border-[var(--color-line-strong)] accent-[var(--color-green)]"
              />
              <span className="min-w-0 break-words">{option.label}</span>
            </span>
            <span className="shrink-0 text-xs text-[var(--color-muted)]">
              {option.count}
            </span>
          </label>
        ))}
      </div>
    </FilterGroupShell>
  );
}

export function PriceRangeSlider({
  id,
  minPaise,
  maxPaise,
  selectedMinPaise,
  selectedMaxPaise,
  onChange,
}: {
  id: string;
  minPaise: number;
  maxPaise: number;
  selectedMinPaise: number;
  selectedMaxPaise: number;
  onChange: (nextMinPaise: number, nextMaxPaise: number) => void;
}) {
  const range = Math.max(1, maxPaise - minPaise);
  const left = ((selectedMinPaise - minPaise) / range) * 100;
  const right = 100 - ((selectedMaxPaise - minPaise) / range) * 100;
  const step = 100;
  const disabled = minPaise >= maxPaise;

  const updateMin = (value: string) => {
    const next = Number.parseInt(value, 10);
    if (!Number.isFinite(next)) return;
    onChange(Math.min(next, selectedMaxPaise), selectedMaxPaise);
  };
  const updateMax = (value: string) => {
    const next = Number.parseInt(value, 10);
    if (!Number.isFinite(next)) return;
    onChange(selectedMinPaise, Math.max(next, selectedMinPaise));
  };

  return (
    <FilterGroupShell title="Price">
      <div className="mt-3">
        <p className="text-sm font-medium text-[var(--color-ink)]">
          {formatInrFromPaise(selectedMinPaise)} - {formatInrFromPaise(selectedMaxPaise)}
        </p>
        <div className="relative mt-3 h-10">
          <div className="absolute inset-x-0 top-1/2 h-1 -translate-y-1/2 rounded-full bg-[var(--color-line)]" />
          <div
            className="absolute top-1/2 h-1 -translate-y-1/2 rounded-full bg-[var(--color-green)]"
            style={{ left: `${left}%`, right: `${right}%` }}
          />
          <input
            id={`${id}-min`}
            type="range"
            min={minPaise}
            max={maxPaise}
            step={step}
            value={selectedMinPaise}
            disabled={disabled}
            onChange={(event) => updateMin(event.target.value)}
            aria-label="Minimum price"
            aria-valuetext={formatInrFromPaise(selectedMinPaise)}
            className={cn(
              "storefront-range-input absolute inset-x-0 top-0 h-10 w-full bg-transparent",
              selectedMinPaise >= selectedMaxPaise && "z-20"
            )}
          />
          <input
            id={`${id}-max`}
            type="range"
            min={minPaise}
            max={maxPaise}
            step={step}
            value={selectedMaxPaise}
            disabled={disabled}
            onChange={(event) => updateMax(event.target.value)}
            aria-label="Maximum price"
            aria-valuetext={formatInrFromPaise(selectedMaxPaise)}
            className="storefront-range-input absolute inset-x-0 top-0 z-10 h-10 w-full bg-transparent"
          />
        </div>
        <div className="mt-1 flex justify-between text-xs text-[var(--color-muted)]">
          <span>{formatInrFromPaise(minPaise)}</span>
          <span>{formatInrFromPaise(maxPaise)}</span>
        </div>
      </div>
    </FilterGroupShell>
  );
}
