"use client";

import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { HeroHeading, Kicker } from "@/components/ui/typography";
import { useHeaderHeight } from "@/hooks/use-header-height";
import { cn } from "@/lib/utils";

function useIsMobile(breakpointPx = 767): boolean {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const query = window.matchMedia(`(max-width: ${breakpointPx}px)`);
    const update = () => setIsMobile(query.matches);
    update();
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  }, [breakpointPx]);

  return isMobile;
}

const CROSSFADE_SEC = 0.6;

/**
 * Plays a sequence of clips on a loop with a brief crossfade between each one. Uses two stacked
 * <video> elements ("slots") and swaps which is on top — the back slot is preloaded and starts
 * playing slightly before the front slot ends, so both play simultaneously during the fade.
 */
function PlaylistVideo({ playlist }: { playlist: string[] }) {
  const [slotItem, setSlotItem] = useState<[number, number]>([0, playlist.length > 1 ? 1 : 0]);
  const [front, setFront] = useState<0 | 1>(0);
  const videoRefs = useRef<[HTMLVideoElement | null, HTMLVideoElement | null]>([null, null]);
  const transitioningRef = useRef(false);
  const frontRef = useRef<0 | 1>(0);

  useEffect(() => {
    frontRef.current = front;
    videoRefs.current[front]?.play().catch(() => {});
  }, [front]);

  const triggerCrossfade = (slot: 0 | 1) => {
    if (transitioningRef.current || slot !== frontRef.current) return;
    transitioningRef.current = true;
    const otherSlot: 0 | 1 = slot === 0 ? 1 : 0;
    videoRefs.current[otherSlot]?.play().catch(() => {});
    setFront(otherSlot);
  };

  const handleTimeUpdate = (slot: 0 | 1) => {
    const video = videoRefs.current[slot];
    if (!video || !video.duration) return;
    if (video.duration - video.currentTime <= CROSSFADE_SEC) triggerCrossfade(slot);
  };

  const handleEnded = (slot: 0 | 1) => {
    // This slot just finished and is now in the background — load it with the clip that comes
    // after whatever the other (now-front) slot is showing, so it's ready for its next turn.
    setSlotItem((prev) => {
      const otherSlot: 0 | 1 = slot === 0 ? 1 : 0;
      const next = [...prev] as [number, number];
      next[slot] = (prev[otherSlot] + 1) % playlist.length;
      return next;
    });
    transitioningRef.current = false;
  };

  return (
    <>
      {([0, 1] as const).map((slot) => (
        <video
          key={`slot-${slot}-${slotItem[slot]}`}
          ref={(el) => {
            videoRefs.current[slot] = el;
          }}
          className="absolute inset-0 h-full w-full object-cover transition-opacity ease-linear"
          style={{ opacity: front === slot ? 1 : 0, transitionDuration: `${CROSSFADE_SEC * 1000}ms` }}
          muted
          playsInline
          preload="auto"
          aria-hidden="true"
          onTimeUpdate={() => handleTimeUpdate(slot)}
          onEnded={() => handleEnded(slot)}
        >
          <source src={playlist[slotItem[slot]]} type="video/mp4" />
        </video>
      ))}
    </>
  );
}

export interface FullBleedVideoSectionProps {
  id?: string;
  src: string;
  /** Optional alternate source served on small screens (e.g. a portrait-cropped cut of the same clip). */
  mobileSrc?: string;
  /** Sequence of clips played one after another (looping back to the first) on small screens. Takes precedence over mobileSrc. */
  mobilePlaylist?: string[];
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
  mobileSrc,
  mobilePlaylist,
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
  const isMobile = useIsMobile();
  const usePlaylist = isMobile && !!mobilePlaylist && mobilePlaylist.length > 0;

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
        {usePlaylist ? (
          <PlaylistVideo playlist={mobilePlaylist!} />
        ) : (
          <video
            className="absolute inset-0 h-full w-full object-cover"
            autoPlay
            muted
            loop
            playsInline
            preload="metadata"
            aria-hidden="true"
          >
            {mobileSrc ? <source media="(max-width: 767px)" src={mobileSrc} type="video/mp4" /> : null}
            <source src={src} type="video/mp4" />
          </video>
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-[var(--color-deep)]/80 via-[var(--color-deep)]/10 to-transparent" />
        <div
          className={cn(
            "absolute inset-x-0 bottom-0 px-6 pt-8 text-center sm:px-12 sm:pt-12 lg:pb-12",
            "pb-[calc(5.5rem+env(safe-area-inset-bottom))]",
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
