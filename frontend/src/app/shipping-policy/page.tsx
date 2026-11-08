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
      intro="We ship across India. At checkout, you'll see live delivery options and pricing for your exact address — your order is only confirmed once payment and stock are both ready."
      sections={[
        {
          heading: "When your order ships",
          body: [
            "You can cancel for free within 12 hours of placing your order (see our Cancellation Policy) — we hold off on booking your shipment during this window.",
            `After that, we book your shipment and it's usually picked up by the courier within ${normalizedPickupDelayHours} hours, unless we need to reach out about your address.`,
          ],
        },
        {
          heading: "Delivery charges",
          body: [
            "Your delivery charge is worked out live at checkout, based on your address and the courier available for your pin code.",
            "The price you see at checkout is what you pay — we won't switch to a different, pricier courier afterward. If we can't get a valid delivery quote, checkout won't let you proceed.",
          ],
        },
        {
          heading: "Tracking your order",
          body: [
            "Once your order ships, track it under Profile > Orders — you'll see live courier updates there.",
            "If your delivery is delayed, contact support and we'll help you understand what's happening and when to expect it.",
          ],
        },
      ]}
    />
  );
}
