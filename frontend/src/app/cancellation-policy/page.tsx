import { PolicyPageShell } from "@/components/policy-page-shell";

export default function CancellationPolicyPage() {
  return (
    <PolicyPageShell
      eyebrow="Cancellation Policy"
      title="Cancellation is available during eligible fulfillment stages."
      intro="Sudatta's keeps cancellation windows aligned with payment verification and logistics progression so updates stay accurate for customers and support teams."
      sections={[
        {
          heading: "When cancellation is available",
          body: [
            "Cancellation is generally available before pickup or shipment booking milestones complete for your order.",
            "Your order detail timeline shows the latest eligibility state. If cancellation is available, use the in-account action or contact support for assistance.",
          ],
        },
        {
          heading: "After pickup or shipment progress",
          body: [
            "After courier pickup or shipment progression reaches non-cancellable stages, direct cancellation may not be available.",
            "In those cases, resolution may continue through support and courier return-to-origin or post-delivery workflows where applicable.",
          ],
        },
        {
          heading: "Prepaid and refund processing",
          body: [
            "For prepaid orders, refund initiation and settlement depend on payment/refund processing and provider confirmation timelines.",
            "Refund status appears on your order updates once confirmed through the payment and order systems.",
          ],
        },
      ]}
    />
  );
}
