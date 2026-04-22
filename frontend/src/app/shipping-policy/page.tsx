import { PolicyPageShell } from "@/components/policy-page-shell";

export default function ShippingPolicyPage() {
  const pickupDelayHours = Number.parseInt(
    (process.env.PICKUP_DELAY_HOURS ?? "48").trim(),
    10
  );
  const normalizedPickupDelayHours =
    Number.isFinite(pickupDelayHours) && pickupDelayHours > 0
      ? pickupDelayHours
      : 48;

  return (
    <PolicyPageShell
      eyebrow="Shipping Policy"
      title="Delivery designed for special-occasion confidence."
      intro="Sudatta's ships across India with a courier selected at checkout from the live serviceable options available for your delivery address. We only confirm an order once payment and stock allocation succeed."
      sections={[
        {
          heading: "Dispatch timeline",
          body: [
            "Paid orders enter a cancellation window before shipment booking. This enables clean full or partial cancellation without courier-side update dependencies.",
            `After the cancellation window closes, shipment is booked and pickup is scheduled with a ${normalizedPickupDelayHours}-hour operational delay unless we contact you for an address or availability clarification.`,
          ],
        },
        {
          heading: "Shipping charges",
          body: [
            "Shipping is quoted live during checkout based on your address, selected items, and the courier available for your pin code.",
            "We do not silently substitute a different courier at a different price after you pay. If a live quote cannot be confirmed, checkout is blocked until a valid quote is available.",
          ],
        },
        {
          heading: "Tracking and delivery updates",
          body: [
            "Courier movement and milestone updates appear in your order timeline under Profile > Orders once a shipment is booked.",
            "If your shipment experiences an operational delay, our support team can help interpret the latest courier scan and next expected delivery step.",
          ],
        },
      ]}
    />
  );
}
