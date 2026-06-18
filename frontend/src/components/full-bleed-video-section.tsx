"use client";

import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { HeroHeading, Kicker } from "@/components/ui/typography";
import { useHeaderHeight } from "@/hooks/use-header-height";
import { cn } from "@/lib/utils";

export interface FullBleedVideoSectionProps {
  id?: string;
  src: string;
  kicker?: string;
  heading: string;
  body?: string;
  ctaLabel?: string;
  onCtaClick?: () => void;
  /** Which side the text panel sits on (and aligns toward) over the video. */
  align?: "left" | "right";
}

export function FullBleedVideoSection({
  id,
  src,
  kicker,
  heading,
  body,
  ctaLabel,
  onCtaClick,
  align = "left",
}: FullBleedVideoSectionProps) {
  const windowRef = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(false);
  const headerHeight = useHeaderHeight();
  const isRight = align === "right";

  useEffect(() => {
    const el = windowRef.current;
    if (!el) return;
    const observer = new IntersectionObserver(([entry]) => setVisible(entry.isIntersecting), {
      threshold: 0,
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return (
    <div id={id}>
      {/* The video+content are truly fixed to the viewport (below the sticky header) and never
          move. Visibility is toggled by an IntersectionObserver watching the "window" div below,
          fading in/out as it scrolls through view. The next section in the page always has its own
          opaque background, which safely absorbs the unavoidable one-viewport overlap as this
          fades out. */}
      <div
        className="pointer-events-none fixed inset-x-0 bottom-0 -z-10 overflow-hidden transition-opacity duration-500 ease-out"
        style={{ top: headerHeight, opacity: visible ? 1 : 0 }}
      >
        <video
          className="absolute inset-0 h-full w-full object-cover"
          autoPlay
          muted
          loop
          playsInline
          preload="metadata"
          aria-hidden="true"
        >
          <source src={src} type="video/mp4" />
        </video>
        <div className="absolute inset-0 bg-gradient-to-t from-[var(--color-deep)]/80 via-[var(--color-deep)]/10 to-transparent" />
        <div
          className={cn(
            "absolute inset-x-0 bottom-0 px-6 py-8 text-center sm:px-12 sm:py-12",
            isRight ? "md:text-right" : "md:text-left"
          )}
        >
          {kicker ? <Kicker tone="inverse">{kicker}</Kicker> : null}
          <HeroHeading
            inverse
            size="sm"
            className={cn("mx-auto max-w-xl", kicker && "mt-3", isRight ? "md:ml-auto md:mr-0" : "md:mx-0")}
          >
            {heading}
          </HeroHeading>
          {body ? (
            <p
              className={cn(
                "mx-auto mt-4 max-w-md text-sm leading-relaxed text-[var(--color-on-deep-muted)] sm:text-base",
                isRight ? "md:ml-auto md:mr-0" : "md:mx-0"
              )}
            >
              {body}
            </p>
          ) : null}
          {ctaLabel ? (
            <Button
              size="lg"
              onClick={onCtaClick}
              className="pointer-events-auto mt-6 rounded-full border-[var(--color-gold)] bg-[var(--color-gold)] text-[var(--color-deep)] hover:border-[var(--color-gold-soft)] hover:bg-[var(--color-gold-soft)]"
            >
              {ctaLabel}
            </Button>
          ) : null}
        </div>
      </div>
      <div ref={windowRef} className="h-[160vh] w-full" aria-hidden="true" />
    </div>
  );
}
