"use client";

import { Component, ErrorInfo, ReactNode } from "react";

type AppErrorBoundaryProps = {
  children: ReactNode;
};

type AppErrorBoundaryState = {
  hasError: boolean;
};

export class AppErrorBoundary extends Component<
  AppErrorBoundaryProps,
  AppErrorBoundaryState
> {
  state: AppErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): AppErrorBoundaryState {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    if (process.env.NODE_ENV !== "production") {
      // eslint-disable-next-line no-console
      console.error("AppErrorBoundary caught error", error, info);
    }
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex min-h-screen items-center justify-center bg-[var(--color-ivory)] px-4">
          <div className="max-w-sm text-center text-sm text-[var(--color-muted)]">
            <p className="font-semibold text-[var(--color-ink)]">
              Something went wrong.
            </p>
            <p className="mt-2">
              Please try reloading the page. If the problem persists, contact
              support.
            </p>
            <button
              type="button"
              onClick={() => window.location.reload()}
              className="mt-4 inline-flex items-center justify-center rounded-full bg-[var(--color-ink)] px-5 py-2 text-xs font-semibold text-white hover:bg-[var(--color-ink)]/90"
            >
              Reload
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

