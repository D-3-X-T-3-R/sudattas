import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";
import { DashboardStats } from "@/components/dashboard-stats";

export default function AdminDashboardPage() {
  return (
    <Section compact className="max-w-6xl">
      <Kicker className="text-[var(--color-muted)]">Overview</Kicker>
      <SectionHeading size="default" className="mt-2">
        At a glance
      </SectionHeading>

      <DashboardStats />

      <Card className="mt-10 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Get started</CardTitle>
        <CardContent className="mt-2">
          <div className="space-y-3 text-sm leading-relaxed text-[var(--color-muted)]">
            <p>
              Stats above are loaded from your GraphQL backend. Ensure{" "}
              <code className="rounded bg-[var(--color-line)]/50 px-1 py-0.5 font-mono text-xs">
                NEXT_PUBLIC_GRAPHQL_URL
              </code>{" "}
              points to the gateway and you’re signed in with an allowed admin account.
            </p>
            <ul className="list-disc space-y-1 pl-5">
              <li>
                Orders today and Revenue (MTD) use <code className="rounded bg-[var(--color-line)]/50 px-1 py-0.5 font-mono text-xs">searchOrder</code> with date filters.
              </li>
              <li>
                Products count uses <code className="rounded bg-[var(--color-line)]/50 px-1 py-0.5 font-mono text-xs">searchProduct</code>.
              </li>
              <li>
                Customers shows &quot;—&quot; until the API exposes a user count or list.
              </li>
            </ul>
          </div>
        </CardContent>
      </Card>
    </Section>
  );
}
