"use client";

import { usePathname, useRouter } from "next/navigation";
import { useSession } from "next-auth/react";
import { useEffect } from "react";
import { AdminShell } from "@/components/admin-shell";
import { AdminAuthSync } from "@/components/admin-auth-sync";

const LOGIN_PATH = "/imtheboss/login";

export default function ImTheBossLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  const pathname = usePathname() ?? "";
  const router = useRouter();
  const { data: session, status } = useSession();

  const isLoginPage = pathname === LOGIN_PATH;

  useEffect(() => {
    if (status === "loading") return;
    if (isLoginPage) return;
    if (!session) {
      router.replace(LOGIN_PATH);
    }
  }, [session, status, isLoginPage, router]);

  if (isLoginPage) {
    return <>{children}</>;
  }

  if (status === "loading" || !session) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-[var(--color-ivory)]">
        <p className="text-sm text-[var(--color-muted)]">Checking sign-in…</p>
      </div>
    );
  }

  return (
    <>
      <AdminAuthSync />
      <AdminShell>{children}</AdminShell>
    </>
  );
}
