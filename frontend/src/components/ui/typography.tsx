"use client";

import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const heroHeadingVariants = cva(
  "font-display font-medium tracking-tight leading-[1.05] text-[var(--color-ink)]",
  {
    variants: {
      size: {
        default: "text-4xl md:text-6xl",
        sm: "text-3xl md:text-5xl",
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
  "font-display font-medium tracking-tight text-[var(--color-ink)]",
  {
    variants: {
      size: {
        default: "text-2xl md:text-3xl",
        lg: "text-3xl md:text-4xl",
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

const kickerVariants = cva("text-xs uppercase tracking-[0.18em] font-semibold", {
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
