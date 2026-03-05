"use client";

import { useRef } from "react";
import { motion, useInView, useReducedMotion } from "framer-motion";
import { cn } from "@/lib/utils";

const MOTION_DURATION = 0.5;
const MOTION_OFFSET = 10;

interface ScrollRevealProps {
  children: React.ReactNode;
  className?: string;
  delay?: number;
  direction?: "up" | "down" | "left" | "right";
}

const directionOffset = {
  up: { y: MOTION_OFFSET },
  down: { y: -MOTION_OFFSET },
  left: { x: MOTION_OFFSET },
  right: { x: -MOTION_OFFSET },
};

export function ScrollReveal({
  children,
  className = "",
  delay = 0,
  direction = "up",
}: ScrollRevealProps) {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { once: true, margin: "0px 0px -40px 0px" });
  const reduceMotion = useReducedMotion();
  const offset = directionOffset[direction];

  const initial = { opacity: 0, ...offset };
  const animate = inView ? { opacity: 1, x: 0, y: 0 } : initial;
  const transition = {
    duration: reduceMotion ? 0 : MOTION_DURATION,
    ease: "easeOut" as const,
    delay,
  };

  return (
    <motion.div
      ref={ref}
      initial={initial}
      animate={animate}
      transition={transition}
      className={cn(className)}
    >
      {children}
    </motion.div>
  );
}
