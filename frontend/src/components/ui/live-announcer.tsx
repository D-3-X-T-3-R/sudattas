"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

type Politeness = "polite" | "assertive";

type LiveAnnouncerContextValue = {
  announce: (message: string, politeness?: Politeness) => void;
};

const LiveAnnouncerContext = createContext<LiveAnnouncerContextValue | null>(null);

export function LiveAnnouncerProvider({ children }: { children: React.ReactNode }) {
  const [politeMessage, setPoliteMessage] = useState("");
  const [assertiveMessage, setAssertiveMessage] = useState("");
  const pendingTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (pendingTimerRef.current) {
        clearTimeout(pendingTimerRef.current);
        pendingTimerRef.current = null;
      }
    };
  }, []);

  const schedule = useCallback((fn: () => void) => {
    if (pendingTimerRef.current) {
      clearTimeout(pendingTimerRef.current);
    }
    pendingTimerRef.current = setTimeout(() => {
      pendingTimerRef.current = null;
      fn();
    }, 20);
  }, []);

  const announce = useCallback((message: string, politeness: Politeness = "polite") => {
    const text = message.trim();
    if (!text) return;
    if (politeness === "assertive") {
      setAssertiveMessage("");
      schedule(() => setAssertiveMessage(text));
      return;
    }
    setPoliteMessage("");
    schedule(() => setPoliteMessage(text));
  }, [schedule]);

  const value = useMemo(() => ({ announce }), [announce]);

  return (
    <LiveAnnouncerContext.Provider value={value}>
      {children}
      <div className="sr-only" aria-live="polite" aria-atomic="true">
        {politeMessage}
      </div>
      <div className="sr-only" aria-live="assertive" aria-atomic="true">
        {assertiveMessage}
      </div>
    </LiveAnnouncerContext.Provider>
  );
}

export function useLiveAnnouncer(): LiveAnnouncerContextValue {
  const ctx = useContext(LiveAnnouncerContext);
  if (!ctx) {
    throw new Error("useLiveAnnouncer must be used within LiveAnnouncerProvider");
  }
  return ctx;
}
