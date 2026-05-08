"use client";

import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const heroHeadingVariants = cva(
  "font-display font-semibold tracking-[-0.02em] leading-[1.15] text-[var(--color-ink)]",
  {
    variants: {
      size: {
        default: "text-[2rem] sm:text-[2.4rem] md:text-[3rem]",
        sm: "text-[1.8rem] sm:text-[2.2rem] md:text-[2.8rem]",
      },
      inverse: {
        true: "text-white",
        false: "",
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
  "font-display font-medium tracking-[-0.01em] text-[var(--color-ink)]",
  {
    variants: {
      size: {
        default: "text-[1.75rem] md:text-[2.1rem]",
        lg: "text-[2rem] md:text-[2.45rem]",
      },
    },
    defaultVariants: {
      size: "default",
    },
  }
);

export interface SectionHeadingProps
  extends React.HTMLAttributes<HTMLHeadingElement>,
    VariantProps<typeof sectionHeadingVariants> {}

export function SectionHeading({
  className,
  size,
  ...props
}: SectionHeadingProps) {
  return (
    <h2
      className={cn(sectionHeadingVariants({ size }), className)}
      {...props}
    />
  );
}

const kickerVariants = cva("text-[11px] uppercase tracking-[0.2em] font-semibold", {
  variants: {
    tone: {
      default: "text-[var(--color-muted)]",
      accent: "text-[var(--color-accent-gold)]",
      inverse: "text-white/90",
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
