"use client";

import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const heroHeadingVariants = cva(
  "font-display font-semibold tracking-[-0.02em] leading-[1.08]",
  {
    variants: {
      size: {
        default: "text-[2.5rem] sm:text-[3.4rem] md:text-[4.5rem] lg:text-[5.25rem]",
        sm: "text-[2rem] sm:text-[2.6rem] md:text-[3.4rem] lg:text-[3.9rem]",
      },
      inverse: {
        true: "text-[var(--color-on-deep)]",
        false: "text-[var(--color-ink)]",
      },
    },
    defaultVariants: {
      size: "default",
      inverse: false,
    },
  }
);

export interface HeroHeadingProps
  extends React.HTMLAttributes<HTMLHeadingElement>,
    VariantProps<typeof heroHeadingVariants> {}

export function HeroHeading({
  className,
  size,
  inverse,
  ...props
}: HeroHeadingProps) {
  return (
    <h1
      className={cn(heroHeadingVariants({ size, inverse }), className)}
      {...props}
    />
  );
}

const sectionHeadingVariants = cva(
  "font-display font-medium tracking-[-0.01em] leading-[1.12] text-[var(--color-ink)]",
  {
    variants: {
      size: {
        default: "text-[1.9rem] md:text-[2.4rem] lg:text-[2.65rem]",
        lg: "text-[2.2rem] md:text-[2.75rem] lg:text-[3.25rem]",
      },
      inverse: {
        true: "text-[var(--color-on-deep)]",
        false: "",
      },
    },
    defaultVariants: {
      size: "default",
      inverse: false,
    },
  }
);

export interface SectionHeadingProps
  extends React.HTMLAttributes<HTMLHeadingElement>,
    VariantProps<typeof sectionHeadingVariants> {}

export function SectionHeading({
  className,
  size,
  inverse,
  ...props
}: SectionHeadingProps) {
  return (
    <h2
      className={cn(sectionHeadingVariants({ size, inverse }), className)}
      {...props}
    />
  );
}

const kickerVariants = cva("inline-flex items-center gap-2 text-[11px] uppercase tracking-[0.24em] font-semibold", {
  variants: {
    tone: {
      default: "text-[var(--color-muted)]",
      accent: "text-[var(--color-accent-gold)]",
      inverse: "text-[var(--color-on-deep-muted)]",
    },
  },
  defaultVariants: {
    tone: "default",
  },
});

export interface KickerProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof kickerVariants> {}

export function Kicker({ className, tone, ...props }: KickerProps) {
  return (
    <span
      className={cn(kickerVariants({ tone }), className)}
      {...props}
    />
  );
}
