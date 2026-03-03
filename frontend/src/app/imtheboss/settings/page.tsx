import { Card, CardContent } from "@/components/ui/card";

export default function AdminSettingsPage() {
  return (
    <Card>
      <CardContent className="py-8 text-center">
        <p className="text-[var(--color-muted)]">
          Settings — connect to your backend.
        </p>
      </CardContent>
    </Card>
  );
}
