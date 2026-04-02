"use client";

import { usePathname } from "next/navigation";
import { AdminShell } from "@/components/admin-shell";
import { AppErrorBoundary } from "@/components/app-error-boundary";

const LOGIN_PATH = "/imtheboss/login";

export default function ImTheBossLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  const pathname = usePathname() ?? "";
  if (pathname === LOGIN_PATH) {
    return <>{children}</>;
  }

  return (
    <AppErrorBoundary>
      <div className="admin-root min-h-screen">
        <AdminShell>{children}</AdminShell>
      </div>
    </AppErrorBoundary>
  );
}
