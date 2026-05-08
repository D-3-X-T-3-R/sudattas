import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { DashboardStats } from "@/components/dashboard-stats";
import { DashboardCharts } from "@/components/dashboard-charts";
import { DashboardObservability } from "@/components/dashboard-observability";

export default function AdminDashboardPage() {
  return (
    <AdminPageShell
      label="Overview"
      title="Welcome back"
      description="Track store performance, order health, and operational risk at a glance."
    >
      <DashboardStats />
      <DashboardCharts />
      <DashboardObservability />
    </AdminPageShell>
  );
}
