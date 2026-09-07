"use client";

import { createContext, useContext, useMemo, useState } from "react";
import { signIn } from "next-auth/react";
import Image from "next/image";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Kicker, SectionHeading } from "@/components/ui/typography";
import { trackClientTelemetry } from "@/lib/client-telemetry";

type StorefrontLoginContextValue = {
  openLogin: (callbackUrl?: string) => void;
};

type LoginDialogProps = {
  open: boolean;
  busy: boolean;
  onOpenChange: (open: boolean) => void;
  onGoogleSignIn: () => void;
};

const StorefrontLoginContext = createContext<StorefrontLoginContextValue>({
  openLogin: () => undefined,
});

/** Official Google "G" mark, per Google's branding guidelines for sign-in buttons. */
function GoogleGlyph() {
  return (
    <svg viewBox="0 0 48 48" aria-hidden="true" className="h-4 w-4 shrink-0">
      <path
        fill="#4285F4"
        d="M45.12 24.5c0-1.56-.14-3.06-.4-4.5H24v8.51h11.84c-.51 2.75-2.06 5.08-4.39 6.64v5.52h7.11c4.16-3.83 6.56-9.47 6.56-16.17z"
      />
      <path
        fill="#34A853"
        d="M24 46c5.94 0 10.92-1.97 14.56-5.33l-7.11-5.52c-1.97 1.32-4.49 2.1-7.45 2.1-5.73 0-10.58-3.87-12.31-9.07H4.34v5.7C7.96 41.07 15.4 46 24 46z"
      />
      <path
        fill="#FBBC05"
        d="M11.69 28.18C11.25 26.86 11 25.45 11 24s.25-2.86.69-4.18v-5.7H4.34C2.85 17.09 2 20.45 2 24s.85 6.91 2.34 9.88l7.35-5.7z"
      />
      <path
        fill="#EA4335"
        d="M24 10.75c3.23 0 6.13 1.11 8.41 3.29l6.31-6.31C34.91 4.18 29.93 2 24 2 15.4 2 7.96 6.93 4.34 14.12l7.35 5.7c1.73-5.2 6.58-9.07 12.31-9.07z"
      />
    </svg>
  );
}

function StorefrontLoginDialog({
  open,
  busy,
  onOpenChange,
  onGoogleSignIn,
}: LoginDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-sm"
        contentClassName="flex flex-col items-center px-8 py-10 text-center sm:px-10"
        aria-describedby="login-dialog-desc"
      >
        <Image
          src="/hero/sudattas-logo.png"
          alt="Sudatta's Designer Boutique"
          width={148}
          height={88}
          priority
          unoptimized
          className="h-12 w-auto object-contain"
        />

        <Kicker tone="accent" className="mt-7">
          Welcome
        </Kicker>
        <SectionHeading className="mt-2 text-[1.6rem] leading-tight md:text-[1.85rem]">
          Sign in to continue
        </SectionHeading>
        <p
          id="login-dialog-desc"
          className="mt-3 max-w-[26rem] text-sm leading-relaxed text-[var(--color-muted)]"
        >
          Sign in with Google to track orders, save your wishlist, and check out faster.
        </p>

        <Button
          type="button"
          size="lg"
          disabled={busy}
          className="mt-8 w-full gap-2.5 bg-white text-[var(--color-ink)] hover:bg-[var(--color-surface-soft)] hover:text-[var(--color-ink)]"
          variant="outline"
          onClick={onGoogleSignIn}
        >
          <GoogleGlyph />
          Continue with Google
        </Button>

        <p className="mt-6 text-xs leading-relaxed text-[var(--color-muted)]">
          By continuing, you agree to Sudatta&apos;s Terms and Privacy Policy.
        </p>
      </DialogContent>
    </Dialog>
  );
}

export function StorefrontLoginProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const [open, setOpen] = useState(false);
  const [callbackUrl, setCallbackUrl] = useState<string | undefined>(undefined);
  const [busy, setBusy] = useState(false);

  const openLogin = (nextCallbackUrl?: string) => {
    setCallbackUrl(nextCallbackUrl);
    setOpen(true);
  };

  async function handleGoogleSignIn() {
    setBusy(true);
    trackClientTelemetry({
      route:
        callbackUrl ||
        (typeof window !== "undefined" ? window.location.pathname : "/"),
      userMode: "public",
      action: "AUTH_SIGN_IN_GOOGLE_ATTEMPT",
      outcome: "success",
      errorClass: "none",
      errorCode: null,
      message: "Storefront Google sign-in initiated from dialog.",
      status: 200,
    });
    try {
      await signIn("google", callbackUrl ? { callbackUrl } : undefined);
    } finally {
      setBusy(false);
    }
  }

  /* ────────────────────────────────────────────────────────────────────────
   * Phone/OTP sign-in — disabled for now (Google is the only sign-in method
   * while this is off). Kept here, commented out, in case it comes back:
   *
   *   import { useLiveAnnouncer } from "@/components/ui/live-announcer";
   *   const { announce } = useLiveAnnouncer();
   *   const [phone, setPhone] = useState("");
   *   const [otp, setOtp] = useState("");
   *   const [otpSent, setOtpSent] = useState(false);
   *   const [error, setError] = useState<string | null>(null);
   *
   *   async function requestOtp() {
   *     const trimmed = phone.trim();
   *     if (!trimmed) {
   *       const msg = "Enter your phone number first.";
   *       setError(msg);
   *       announce(msg, "assertive");
   *       return;
   *     }
   *     setBusy(true);
   *     setError(null);
   *     try {
   *       const res = await fetch("/api/auth/phone-otp/request", {
   *         method: "POST",
   *         headers: { "Content-Type": "application/json" },
   *         body: JSON.stringify({ phone: trimmed }),
   *       });
   *       const body = (await res.json().catch(() => ({}))) as { ok?: boolean; error?: string };
   *       if (!res.ok || body.ok !== true) {
   *         throw new Error(body.error || "Could not send OTP");
   *       }
   *       setOtpSent(true);
   *       const msg = "OTP sent. Check your phone and enter the code.";
   *       announce(msg, "polite");
   *       trackClientTelemetry({
   *         route: "/api/auth/phone-otp/request",
   *         userMode: "public",
   *         action: "AUTH_OTP_SEND",
   *         outcome: "success",
   *         errorClass: "none",
   *         message: msg,
   *         status: 200,
   *       });
   *     } catch (e) {
   *       const msg = e instanceof Error ? e.message : "Could not send OTP";
   *       setError(msg);
   *       announce(`OTP send failed: ${msg}`, "assertive");
   *       trackClientTelemetry({
   *         route: "/api/auth/phone-otp/request",
   *         userMode: "public",
   *         action: "AUTH_OTP_SEND",
   *         outcome: "failure",
   *         errorClass: "retryable",
   *         errorCode: "OTP_SEND_FAILED",
   *         message: msg,
   *         status: 400,
   *       });
   *     } finally {
   *       setBusy(false);
   *     }
   *   }
   *
   *   async function verifyOtpAndSignIn() {
   *     const trimmedPhone = phone.trim();
   *     const trimmedOtp = otp.trim();
   *     if (!trimmedPhone || !trimmedOtp) {
   *       const msg = "Enter phone number and OTP.";
   *       setError(msg);
   *       announce(msg, "assertive");
   *       return;
   *     }
   *     setBusy(true);
   *     setError(null);
   *     try {
   *       const result = await signIn("phone-otp", {
   *         phone: trimmedPhone,
   *         otp: trimmedOtp,
   *         redirect: false,
   *         callbackUrl: callbackUrl || "/",
   *       });
   *       if (!result || result.error) {
   *         throw new Error(result?.error || "OTP verification failed");
   *       }
   *       const successMessage = "Signed in successfully.";
   *       announce(successMessage, "polite");
   *       setOpen(false);
   *       if (callbackUrl) {
   *         window.location.assign(callbackUrl);
   *       } else {
   *         window.location.reload();
   *       }
   *     } catch (e) {
   *       const msg = e instanceof Error ? e.message : "Login failed";
   *       setError(msg);
   *       announce(`Login failed: ${msg}`, "assertive");
   *       trackClientTelemetry({
   *         route: "/api/auth/[...nextauth]",
   *         userMode: "public",
   *         action: "AUTH_SIGN_IN_PHONE_OTP",
   *         outcome: "failure",
   *         errorClass: "unauthorized",
   *         errorCode: "LOGIN_FAILED",
   *         message: msg,
   *         status: 401,
   *       });
   *     } finally {
   *       setBusy(false);
   *     }
   *   }
   *
   * Dialog JSX (phone input, OTP input, "Send OTP" / "Verify OTP and sign in"
   * buttons, and the live-region status/error paragraphs) previously sat below
   * the Google button in StorefrontLoginDialog. Re-enabling: restore those
   * fields plus `phone`/`otp`/`otpSent`/`error` state and pass them through as
   * LoginDialogProps, same shape as before this change.
   * ──────────────────────────────────────────────────────────────────────── */

  const value = useMemo<StorefrontLoginContextValue>(
    () => ({ openLogin }),
    []
  );

  return (
    <StorefrontLoginContext.Provider value={value}>
      {children}
      <StorefrontLoginDialog
        open={open}
        busy={busy}
        onOpenChange={setOpen}
        onGoogleSignIn={() => void handleGoogleSignIn()}
      />
    </StorefrontLoginContext.Provider>
  );
}

export function useStorefrontLogin(): StorefrontLoginContextValue {
  return useContext(StorefrontLoginContext);
}
