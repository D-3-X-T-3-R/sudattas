import { Star } from "lucide-react";
import { cn } from "@/lib/utils";

export function Rating({ value }: { value: number }) {
  const full = Math.floor(value);
  const half = value - full >= 0.5;
  return (
    <div className="flex items-center gap-1">
      {Array.from({ length: 5 }).map((_, i) => {
        const filled = i < full;
        const isHalf = i === full && half;
        return (
          <span key={i} aria-hidden className="inline-flex">
            <Star
              className={cn(
                "h-4 w-4",
                filled || isHalf ? "text-[var(--color-ink)]" : "text-[#CFC7B8]"
              )}
              fill={filled || isHalf ? "currentColor" : "none"}
              style={{
                clipPath: isHalf
                  ? "polygon(0 0, 50% 0, 50% 100%, 0 100%)"
                  : undefined,
              }}
            />
          </span>
        );
      })}
      <span className="ml-1 text-xs text-[var(--color-muted)]">
        {value.toFixed(1)}
      </span>
    </div>
  );
}
