import { PolicyPageShell } from "@/components/policy-page-shell";

export default function CancellationPolicyPage() {
  return (
    <PolicyPageShell
      eyebrow="Cancellation Policy"
      title="You can cancel your order within 12 hours of placing it."
      intro="We keep our cancellation window simple, and your order page always shows whether cancellation is still available."
      sections={[
        {
          heading: "Cancelling within 12 hours",
          body: [
            "You can cancel your order yourself, free of charge, within 12 hours of placing it — just use the cancel option on your order page.",
            "Once you cancel, we stop the order before it ships and start your refund right away.",
          ],
        },
        {
          heading: "After the 12-hour window",
          body: [
            "Once the window closes and your order is on its way, you can no longer cancel it yourself.",
            "If you still don't want the order, you can refuse it at delivery, or contact support once it arrives — see our Returns & Exchanges page for what happens next.",
          ],
        },
        {
          heading: "Refunds for prepaid orders",
          body: [
            "If you paid online, your refund goes back to your original payment method once your cancellation is processed.",
            "You'll see your refund status right on your order page as soon as it's confirmed.",
          ],
        },
      ]}
    />
  );
}
