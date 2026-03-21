"use client";

import { Component, ErrorInfo, ReactNode } from "react";

type AppErrorBoundaryProps = { children: ReactNode };
type AppErrorBoundaryState = { hasError: boolean };

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
      console.error("AppErrorBoundary caught error", error, info);
    }
  }

  render() {
    if (this.state.hasError) {
      return (
        <>
          <style>{`
            @keyframes eb-ripple {
              0%   { transform: scale(0.6); opacity: 0.7; }
              100% { transform: scale(2.2); opacity: 0; }
            }
            @keyframes eb-spin-slow {
              from { transform: rotate(0deg); }
              to   { transform: rotate(360deg); }
            }
            @keyframes eb-fade-up {
              from { opacity: 0; transform: translateY(18px); }
              to   { opacity: 1; transform: translateY(0); }
            }
            .eb-ripple-1 { animation: eb-ripple 2.4s ease-out infinite; }
            .eb-ripple-2 { animation: eb-ripple 2.4s ease-out 0.7s infinite; }
            .eb-ripple-3 { animation: eb-ripple 2.4s ease-out 1.4s infinite; }
            .eb-spin     { animation: eb-spin-slow 8s linear infinite; }
            .eb-fade-up  { animation: eb-fade-up 0.8s cubic-bezier(0.22,1,0.36,1) both; }
            .eb-fade-up-2 { animation: eb-fade-up 0.8s 0.15s cubic-bezier(0.22,1,0.36,1) both; }
            .eb-fade-up-3 { animation: eb-fade-up 0.8s 0.3s cubic-bezier(0.22,1,0.36,1) both; }
          `}</style>

          <div
            style={{ backgroundColor: "#F5EFE6" }}
            className="relative flex min-h-screen flex-col items-center justify-center overflow-hidden px-4"
          >
            {/* Ripple rings */}
            <div className="relative flex items-center justify-center" style={{ width: 140, height: 140 }}>
              {/* Spinning dashed ring */}
              <div
                className="eb-spin absolute inset-0 rounded-full"
                style={{
                  border: "1px dashed #a68b5b",
                  opacity: 0.35,
                }}
              />
              {/* Ripple layers */}
              <div
                className="eb-ripple-1 absolute rounded-full"
                style={{ width: 56, height: 56, border: "1.5px solid #a68b5b" }}
              />
              <div
                className="eb-ripple-2 absolute rounded-full"
                style={{ width: 56, height: 56, border: "1.5px solid #a68b5b" }}
              />
              <div
                className="eb-ripple-3 absolute rounded-full"
                style={{ width: 56, height: 56, border: "1.5px solid #a68b5b" }}
              />
              {/* Centre dot */}
              <div
                className="relative rounded-full"
                style={{ width: 28, height: 28, background: "#a68b5b", opacity: 0.9 }}
              />
            </div>

            {/* Text */}
            <div className="mt-10 text-center">
              <p
                className="eb-fade-up uppercase tracking-[0.22em] text-xs font-semibold"
                style={{ color: "#a68b5b" }}
              >
                Sudatta&apos;s
              </p>
              <p
                className="eb-fade-up-2 mt-3 font-display text-xl font-medium tracking-tight"
                style={{ color: "#1a1814" }}
              >
                Something came undone.
              </p>
              <p
                className="eb-fade-up-3 mt-2 text-sm"
                style={{ color: "#6b6560" }}
              >
                Try reloading — your bag is still safe.
              </p>
              <div className="eb-fade-up-3 mt-6">
                <button
                  type="button"
                  onClick={() => window.location.reload()}
                  className="inline-flex items-center gap-2 rounded-full px-6 py-2.5 text-xs font-semibold uppercase tracking-[0.18em] text-white transition-opacity hover:opacity-80"
                  style={{ background: "#1a1814" }}
                >
                  Reload
                </button>
              </div>
            </div>
          </div>
        </>
      );
    }

    return this.props.children;
  }
}
