import { SectionHeading } from "@/components/ui/typography";
import { DashboardStats } from "@/components/dashboard-stats";
import { DashboardCharts } from "@/components/dashboard-charts";

export default function AdminDashboardPage() {
  return (
    <div className="mx-auto max-w-6xl w-full">
      <div className="mb-8">
        <p className="text-sm text-[var(--color-muted)]">Overview</p>
        <SectionHeading size="default" className="mt-1">
          Welcome back
        </SectionHeading>
        <p className="mt-1 text-sm leading-relaxed text-[var(--color-muted)]">
          Here’s what’s happening with your store today.
        </p>
      </div>

      <DashboardStats />

      <DashboardCharts />
    </div>
  );
}
