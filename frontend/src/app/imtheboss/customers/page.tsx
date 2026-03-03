import { Card, CardContent, CardTitle } from "@/components/ui/card";

export default function AdminCustomersPage() {
  return (
    <Card className="max-w-2xl">
      <CardTitle>Customers</CardTitle>
      <CardContent className="mt-2 space-y-2 text-sm text-[var(--color-muted)]">
        <p>
          Once wired to your backend, this view will list customers and their
          recent activity.
        </p>
        <p>
          You can later add search, filters (for example, “high value”, “recent”),
          and links into individual customer profiles.
        </p>
      </CardContent>
    </Card>
  );
}
