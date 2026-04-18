import { PolicyPageShell } from "@/components/policy-page-shell";

export default function TermsAndConditionsPage() {
  return (
    <PolicyPageShell
      eyebrow="Terms & Conditions"
      title="Order, payment, and service terms for Sudatta's."
      intro="By placing an order with Sudatta's, you confirm that the delivery details, selected items, and payment confirmation submitted during checkout are accurate and authorized by you."
      sections={[
        {
          heading: "Orders and payment",
          body: [
            "Orders are confirmed only after payment verification succeeds. Inventory is reserved against the specific items you selected during checkout.",
            "Payment verification, refund handling, and shipment events are reconciled through provider callbacks and internal safety checks to avoid duplicate charges, duplicate refunds, or invalid state transitions.",
          ],
        },
        {
          heading: "Fulfillment and cancellation",
          body: [
            "A customer may cancel an order until pickup is completed by the logistics partner. After pickup completion, cancellation is disabled and any return outcome follows the courier return workflow.",
            "If logistics cancellation cannot be completed immediately, the order may remain in a retry-safe cancellation-pending state until the partner confirms the cancellation.",
          ],
        },
        {
          heading: "Support and resolution",
          body: [
            "Support requests should reference the order number shown in your order detail page. We may request delivery images, courier scans, or payment references to resolve a request accurately.",
          ],
        },
      ]}
    />
  );
}
