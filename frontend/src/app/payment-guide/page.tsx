import { PolicyPageShell } from "@/components/policy-page-shell";

export default function PaymentGuidePage() {
  return (
    <PolicyPageShell
      eyebrow="COD & Prepaid Guide"
      title="Choose the payment mode that fits your order."
      intro="Sudatta's supports prepaid and Cash on Delivery (COD) paths where available. Availability can vary by product, delivery location, order value, and operational risk checks."
      sections={[
        {
          heading: "Prepaid orders",
          body: [
            "Prepaid checkout confirms payment through gateway verification before the order moves forward to fulfillment.",
            "If payment is pending or under review, do not place a second order immediately. Wait for the order status update or contact support.",
          ],
        },
        {
          heading: "Cash on Delivery (COD)",
          body: [
            "COD may be offered only for eligible products, locations, and order profiles at checkout.",
            "If COD is unavailable for your combination, choose prepaid to complete the order.",
          ],
        },
        {
          heading: "Payment failures and retries",
          body: [
            "If payment is not completed, you can retry safely from your bag or checkout path.",
            "For bank debit concerns where status is unclear, share the order reference and payment reference with support for guided resolution.",
          ],
        },
      ]}
    />
  );
}
