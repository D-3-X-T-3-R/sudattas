import { Card, CardContent, CardTitle } from "@/components/ui/card";

const STATS = [
  {
    label: "Orders today",
    value: "—",
    sub: "Will show today’s orders once GraphQL is connected.",
  },
  {
    label: "Revenue (MTD)",
    value: "—",
    sub: "Will show month-to-date revenue from completed orders.",
  },
  {
    label: "Products",
    value: "—",
    sub: "Will show total live products in your catalog.",
  },
  {
    label: "Customers",
    value: "—",
    sub: "Will show total registered customers.",
  },
];

export default function AdminDashboardPage() {
  return (
    <>
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {STATS.map((card) => (
          <Card key={card.label}>
            <CardTitle>{card.label}</CardTitle>
            <CardContent className="mt-2">
              <div className="text-2xl font-semibold text-[var(--color-ink)]">
                {card.value}
              </div>
              <div className="mt-1 text-xs text-[var(--color-muted)]">
                {card.sub}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
      <Card className="mt-8">
        <CardTitle>Get started</CardTitle>
        <CardContent className="mt-2">
          <div className="space-y-2 text-sm text-[var(--color-muted)]">
            <p>
              Wire this dashboard to your GraphQL backend to see live orders,
              revenue, products, and customers here.
            </p>
            <ul className="list-disc space-y-1 pl-5">
              <li>
                Confirm <code>NEXT_PUBLIC_GRAPHQL_URL</code> points to the
                GraphQL gateway (e.g. <code>http://localhost:8080/v2</code>).
              </li>
              <li>
                Ensure admin auth is configured (token or session) so metrics
                queries can run.
              </li>
              <li>
                Once wired, replace these placeholders with real summary
                queries.
              </li>
            </ul>
          </div>
        </CardContent>
      </Card>
    </>
  );
}
