import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { DashboardStats } from "@/components/dashboard-stats";
import { DashboardMetrics } from "@/components/dashboard-metrics";
import { DashboardCharts } from "@/components/dashboard-charts";
import { DashboardDonuts } from "@/components/dashboard-donuts";
import { DashboardProductPerformance } from "@/components/dashboard-product-performance";
import { DashboardPayment } from "@/components/dashboard-payment";
import { DashboardObservability } from "@/components/dashboard-observability";

export default function AdminDashboardPage() {
  return (
    <AdminPageShell
      label="Overview"
      title="Welcome back"
      description="Track store performance, order health, and operational risk at a glance."
    >
      <DashboardStats />
      <DashboardMetrics />
      <DashboardCharts />
      <DashboardDonuts />
      <DashboardProductPerformance />
      <DashboardPayment />
      <DashboardObservability />
    </AdminPageShell>
  );
}
