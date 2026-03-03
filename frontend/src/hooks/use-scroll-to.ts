"use client";

export function goTo(id: string, instant = false): void {
  if (id === "top") {
    window.scrollTo({ top: 0, behavior: instant ? "auto" : "smooth" });
    return;
  }
  const el = document.getElementById(id);
  el?.scrollIntoView({ behavior: instant ? "auto" : "smooth" });
}
