"use client";

import { useEffect, useState } from "react";

/** Live height (px) of the sticky site header, kept in sync via ResizeObserver. */
export function useHeaderHeight(): number {
  const [height, setHeight] = useState(0);

  useEffect(() => {
    const header = document.querySelector("header");
    if (!(header instanceof HTMLElement)) return;

    const update = () => setHeight(header.getBoundingClientRect().height);
    update();

    const observer = new ResizeObserver(update);
    observer.observe(header);
    return () => observer.disconnect();
  }, []);

  return height;
}
