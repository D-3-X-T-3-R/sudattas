"use client";

import {
  createContext,
  useCallback,
  useContext,
  useMemo,
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

  const announce = useCallback((message: string, politeness: Politeness = "polite") => {
    const text = message.trim();
    if (!text) return;
    if (politeness === "assertive") {
      setAssertiveMessage("");
      window.setTimeout(() => setAssertiveMessage(text), 20);
      return;
    }
    setPoliteMessage("");
    window.setTimeout(() => setPoliteMessage(text), 20);
  }, []);

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

