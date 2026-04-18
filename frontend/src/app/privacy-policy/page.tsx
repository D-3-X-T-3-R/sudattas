import { PolicyPageShell } from "@/components/policy-page-shell";

export default function PrivacyPolicyPage() {
  return (
    <PolicyPageShell
      eyebrow="Privacy Policy"
      title="We use customer data to fulfill orders, not to overreach."
      intro="Sudatta's stores the information required to authenticate your account, process your order, coordinate delivery, and provide support after purchase."
      sections={[
        {
          heading: "What we collect",
          body: [
            "Account, address, and order details are used to prepare shipments, verify ownership of support requests, and show your order timeline accurately.",
            "Payment verification is handled through Razorpay. We do not expose payment secrets or raw webhook signatures in customer-facing logs or dashboards.",
          ],
        },
        {
          heading: "Operational use",
          body: [
            "Courier and payment providers receive only the information required to complete delivery and refund workflows.",
            "Operational telemetry is used to detect checkout failures, webhook issues, refund failures, and delayed fulfillment states so we can respond quickly when something needs attention.",
          ],
        },
        {
          heading: "Support and account access",
          body: [
            "Order support is restricted to the authenticated account that owns the order or to authorized internal staff. Protected operations derive customer identity from verified session or internal service context.",
          ],
        },
      ]}
    />
  );
}
