import { AdminShell } from "@/components/admin-shell";

export default function ImTheBossLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return <AdminShell>{children}</AdminShell>;
}
