"use client";

import { useRef, useEffect, useState } from "react";
import { cn } from "@/lib/utils";

interface ScrollRevealProps {
  children: React.ReactNode;
  className?: string;
  delay?: number;
  direction?: "up" | "down" | "left" | "right";
}

export function ScrollReveal({
  children,
  className = "",
  delay = 0,
  direction = "up",
}: ScrollRevealProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [inView, setInView] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const obs = new IntersectionObserver(
      ([entry]) => {
        if (entry?.isIntersecting) setInView(true);
      },
      { threshold: 0.1, rootMargin: "0px 0px -40px 0px" }
    );
    obs.observe(el);
    return () => obs.disconnect();
  }, []);

  return (
    <div
      ref={ref}
      className={cn(
        "scroll-reveal",
        inView && "scroll-reveal-in-view",
        direction === "up" && "scroll-reveal--up",
        direction === "down" && "scroll-reveal--down",
        direction === "left" && "scroll-reveal--left",
        direction === "right" && "scroll-reveal--right",
        className
      )}
      style={
        delay > 0
          ? ({ "--scroll-reveal-delay": `${delay}s` } as React.CSSProperties)
          : undefined
      }
    >
      {children}
    </div>
  );
}
