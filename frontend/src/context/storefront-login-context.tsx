"use client";

import { createContext, useContext, useMemo, useState } from "react";
import { signIn } from "next-auth/react";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useLiveAnnouncer } from "@/components/ui/live-announcer";
import { trackClientTelemetry } from "@/lib/client-telemetry";

type StorefrontLoginContextValue = {
  openLogin: (callbackUrl?: string) => void;
};

type LoginDialogProps = {
  open: boolean;
  busy: boolean;
  phone: string;
  otp: string;
  otpSent: boolean;
  error: string | null;
  setPhone: (value: string) => void;
  setOtp: (value: string) => void;
  onOpenChange: (open: boolean) => void;
  onGoogleSignIn: () => void;
  onSendOtp: () => void;
  onVerifyOtp: () => void;
};

const StorefrontLoginContext = createContext<StorefrontLoginContextValue>({
  openLogin: () => undefined,
});

function StorefrontLoginDialog({
  open,
  busy,
  phone,
  otp,
  otpSent,
  error,
  setPhone,
  setOtp,
  onOpenChange,
  onGoogleSignIn,
  onSendOtp,
  onVerifyOtp,
}: LoginDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent title="Sign in" className="max-w-md" contentClassName="space-y-4">
        <p id="login-dialog-desc" className="text-sm text-[var(--color-muted)]">
          Continue with Google or sign in with phone OTP.
        </p>

        <Button type="button" className="w-full rounded-full" onClick={onGoogleSignIn}>
          Continue with Google
        </Button>

        <div className="pt-1">
          <label
            htmlFor="login-phone"
            className="mb-1 block text-xs font-medium text-[var(--color-ink)]"
          >
            Phone number
          </label>
          <input
            id="login-phone"
            value={phone}
            onChange={(e) => setPhone(e.target.value)}
            autoComplete="tel"
            inputMode="tel"
            className="h-11 w-full rounded-xl border border-[var(--color-line)] bg-white px-3 text-sm outline-none focus:border-[var(--color-accent-gold)]"
            aria-describedby={error ? "login-dialog-desc login-error" : "login-dialog-desc"}
            aria-invalid={Boolean(error)}
          />
        </div>

        <div>
          <label
            htmlFor="login-otp"
            className="mb-1 block text-xs font-medium text-[var(--color-ink)]"
          >
            One-time password
          </label>
        </div>
        <div className="flex gap-2">
          <Button
            type="button"
            variant="outline"
            className="flex-1 rounded-full"
            disabled={busy}
            onClick={onSendOtp}
          >
            Send OTP
          </Button>
          <input
            id="login-otp"
            value={otp}
            onChange={(e) => setOtp(e.target.value)}
            inputMode="numeric"
            placeholder="OTP"
            className="h-11 w-28 rounded-xl border border-[var(--color-line)] bg-white px-3 text-sm outline-none focus:border-[var(--color-accent-gold)]"
            aria-describedby={error ? "login-error" : undefined}
            aria-invalid={Boolean(error)}
          />
        </div>

        <Button
          type="button"
          className="w-full rounded-full"
          disabled={busy || !otpSent}
          onClick={onVerifyOtp}
        >
          Verify OTP and sign in
        </Button>

        <p className="sr-only" role="status" aria-live="polite">
          {otpSent ? "OTP sent." : ""}
        </p>
        <p
          id="login-error"
          className="text-sm text-red-700"
          role="alert"
          aria-live="assertive"
        >
          {error ?? ""}
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
  const { announce } = useLiveAnnouncer();
  const [open, setOpen] = useState(false);
  const [callbackUrl, setCallbackUrl] = useState<string | undefined>(undefined);
  const [phone, setPhone] = useState("");
  const [otp, setOtp] = useState("");
  const [otpSent, setOtpSent] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openLogin = (nextCallbackUrl?: string) => {
    setCallbackUrl(nextCallbackUrl);
    setPhone("");
    setOtp("");
    setOtpSent(false);
    setError(null);
    setOpen(true);
  };

  async function handleGoogleSignIn() {
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
    await signIn("google", callbackUrl ? { callbackUrl } : undefined);
  }

  async function requestOtp() {
    const trimmed = phone.trim();
    if (!trimmed) {
      const msg = "Enter your phone number first.";
      setError(msg);
      announce(msg, "assertive");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const res = await fetch("/api/auth/phone-otp/request", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ phone: trimmed }),
      });
      const body = (await res.json().catch(() => ({}))) as { ok?: boolean; error?: string };
      if (!res.ok || body.ok !== true) {
        throw new Error(body.error || "Could not send OTP");
      }
      setOtpSent(true);
      const msg = "OTP sent. Check your phone and enter the code.";
      announce(msg, "polite");
      trackClientTelemetry({
        route: "/api/auth/phone-otp/request",
        userMode: "public",
        action: "AUTH_OTP_SEND",
        outcome: "success",
        errorClass: "none",
        message: msg,
        status: 200,
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Could not send OTP";
      setError(msg);
      announce(`OTP send failed: ${msg}`, "assertive");
      trackClientTelemetry({
        route: "/api/auth/phone-otp/request",
        userMode: "public",
        action: "AUTH_OTP_SEND",
        outcome: "failure",
        errorClass: "retryable",
        errorCode: "OTP_SEND_FAILED",
        message: msg,
        status: 400,
      });
    } finally {
      setBusy(false);
    }
  }

  async function verifyOtpAndSignIn() {
    const trimmedPhone = phone.trim();
    const trimmedOtp = otp.trim();
    if (!trimmedPhone || !trimmedOtp) {
      const msg = "Enter phone number and OTP.";
      setError(msg);
      announce(msg, "assertive");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const result = await signIn("phone-otp", {
        phone: trimmedPhone,
        otp: trimmedOtp,
        redirect: false,
        callbackUrl: callbackUrl || "/",
      });
      if (!result || result.error) {
        throw new Error(result?.error || "OTP verification failed");
      }
      const successMessage = "Signed in successfully.";
      announce(successMessage, "polite");
      setOpen(false);
      if (callbackUrl) {
        window.location.assign(callbackUrl);
      } else {
        window.location.reload();
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Login failed";
      setError(msg);
      announce(`Login failed: ${msg}`, "assertive");
      trackClientTelemetry({
        route: "/api/auth/[...nextauth]",
        userMode: "public",
        action: "AUTH_SIGN_IN_PHONE_OTP",
        outcome: "failure",
        errorClass: "unauthorized",
        errorCode: "LOGIN_FAILED",
        message: msg,
        status: 401,
      });
    } finally {
      setBusy(false);
    }
  }

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
        phone={phone}
        otp={otp}
        otpSent={otpSent}
        error={error}
        setPhone={setPhone}
        setOtp={setOtp}
        onOpenChange={setOpen}
        onGoogleSignIn={() => void handleGoogleSignIn()}
        onSendOtp={() => void requestOtp()}
        onVerifyOtp={() => void verifyOtpAndSignIn()}
      />
    </StorefrontLoginContext.Provider>
  );
}

export function useStorefrontLogin(): StorefrontLoginContextValue {
  return useContext(StorefrontLoginContext);
}
