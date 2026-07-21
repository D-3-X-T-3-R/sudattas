"use client";

import { Suspense, useEffect } from "react";
import Image from "next/image";
import { useSearchParams } from "next/navigation";
import { signIn } from "next-auth/react";
import { Button } from "@/components/ui/button";
import { Kicker, SectionHeading } from "@/components/ui/typography";
import { trackClientTelemetry } from "@/lib/client-telemetry";

function GoogleSignInButton({ label = "Sign in with Google" }: { label?: string }) {
  return (
    <Button
      size="lg"
      className="h-12 w-full"
      onClick={() => {
        trackClientTelemetry({
          route: "/imtheboss/login",
          userMode: "admin",
          action: "AUTH_SIGN_IN_GOOGLE_ATTEMPT",
          outcome: "success",
          errorClass: "none",
          errorCode: null,
          message: "Admin sign-in attempt initiated.",
          status: 200,
        });
        signIn("google", { callbackUrl: "/imtheboss" });
      }}
    >
      {label}
    </Button>
  );
}

function AdminLoginForm() {
  const searchParams = useSearchParams();
  const accessDenied = searchParams.get("error") === "AccessDenied";

  useEffect(() => {
    if (!accessDenied) return;
    trackClientTelemetry({
      route: "/imtheboss/login",
      userMode: "admin",
      action: "AUTH_SIGN_IN_GOOGLE",
      outcome: "failure",
      errorClass: "unauthorized",
      errorCode: "ACCESS_DENIED",
      message: "Admin access denied by backend role check.",
      status: 403,
    });
  }, [accessDenied]);

  return (
    <div className="flex min-h-screen items-center justify-center bg-[var(--color-bg-ivory)] px-4 py-10">
      <section className="w-full max-w-md rounded-2xl border border-[var(--color-line)] bg-[var(--color-surface)] p-9 text-center shadow-[var(--shadow-soft)]">
        <Image
          src="/logo.png"
          alt="Sudatta's"
          width={170}
          height={52}
          className="mx-auto h-9 w-auto"
        />
        <Kicker className="mt-5">Admin Access</Kicker>
        <SectionHeading size="default" className="mt-2">
          {accessDenied ? "Access denied" : "Sign in to continue"}
        </SectionHeading>
        <p className="mt-3 text-[15px] leading-relaxed text-[var(--color-muted)]">
          {accessDenied
            ? "This account doesn't have admin access yet."
            : "Use your Google account to open the Sudatta's admin panel."}
        </p>
        <div className="mt-7">
          <GoogleSignInButton label={accessDenied ? "Try another account" : "Sign in with Google"} />
        </div>
        <p className="mt-4 text-sm text-[var(--color-muted)]">
          You&apos;ll be taken to Google to sign in securely.
        </p>
      </section>
    </div>
  );
}

export default function AdminLoginPage() {
  return (
    <Suspense
      fallback={
        <div className="flex min-h-screen items-center justify-center bg-[var(--color-bg-ivory)]">
          <p className="text-sm text-[var(--color-muted)]">Loading...</p>
        </div>
      }
    >
      <AdminLoginForm />
    </Suspense>
  );
}
