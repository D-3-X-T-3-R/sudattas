"use client";

import { cn } from "@/lib/utils";

export type TimelineStep = {
  label: string;
  detail?: string;
  state: "done" | "current" | "pending";
};

export function StatusTimeline({
  steps,
  className,
}: {
  steps: TimelineStep[];
  className?: string;
}) {
  return (
    <ol className={cn("space-y-3", className)}>
      {steps.map((step, idx) => {
        const done = step.state === "done";
        const current = step.state === "current";
        return (
          <li key={`${step.label}-${idx}`} className="flex gap-3">
            <div className="flex flex-col items-center pt-0.5">
              <span
                className={cn(
                  "h-2.5 w-2.5 rounded-full border",
                  done && "border-[var(--color-green)] bg-[var(--color-green)]",
                  current && "border-[var(--color-gold)] bg-[var(--color-gold)]",
                  step.state === "pending" && "border-[var(--color-line-strong)] bg-white"
                )}
              />
              {idx < steps.length - 1 ? (
                <span className="mt-1 h-full min-h-5 w-px bg-[var(--color-line)]" />
              ) : null}
            </div>
            <div className="pb-2">
              <p
                className={cn(
                  "text-sm font-medium",
                  step.state === "pending" ? "text-[var(--color-muted)]" : "text-[var(--color-ink)]"
                )}
              >
                {step.label}
              </p>
              {step.detail ? (
                <p className="mt-0.5 text-xs text-[var(--color-muted)]">{step.detail}</p>
              ) : null}
            </div>
          </li>
        );
      })}
    </ol>
  );
}
