"use client";

import { useRef } from "react";
import { motion, useInView, useReducedMotion } from "framer-motion";
import { cn } from "@/lib/utils";

const MOTION_OFFSET = 24;

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
  const inView = useInView(ref, { once: true, margin: "0px 0px 180px 0px" });
  const reduceMotion = useReducedMotion();
  const offset = directionOffset[direction];

  const initial = { opacity: 0, ...offset };
  const visible = Boolean(reduceMotion) || inView;
  const animate = visible ? { opacity: 1, x: 0, y: 0 } : initial;

  return (
    <motion.div
      ref={ref}
      initial={initial}
      animate={animate}
      transition={
        reduceMotion
          ? { duration: 0 }
          : {
              opacity: { duration: 0.55, ease: [0.22, 1, 0.36, 1], delay },
              x: { type: "spring", stiffness: 60, damping: 18, delay },
              y: { type: "spring", stiffness: 60, damping: 18, delay },
            }
      }
      className={cn(className)}
    >
      {children}
    </motion.div>
  );
}
