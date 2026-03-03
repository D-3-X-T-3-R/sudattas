import { Card, CardContent, CardTitle } from "@/components/ui/card";

export default function AdminOrdersPage() {
  return (
    <Card className="max-w-2xl">
      <CardTitle>Orders</CardTitle>
      <CardContent className="mt-2 space-y-2 text-sm text-[var(--color-muted)]">
        <p>
          When your backend is connected, recent orders and their statuses will
          appear here.
        </p>
        <p>
          Start by wiring GraphQL queries for orders, then add filters for date
          range, status, and payment state.
        </p>
      </CardContent>
    </Card>
  );
}
