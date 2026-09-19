import { PolicyPageShell } from "@/components/policy-page-shell";

export default function PaymentGuidePage() {
  return (
    <PolicyPageShell
      eyebrow="COD & Prepaid Guide"
      title="Choose the payment mode that fits your order."
      intro="You can pay online (Prepaid), or in cash when your order arrives (Cash on Delivery) — both options are available on every order, at no extra charge."
      sections={[
        {
          heading: "Paying online (Prepaid)",
          body: [
            "We confirm your payment before your order moves to dispatch — this usually takes just a few seconds.",
            "If your payment shows as pending, please wait for a status update rather than placing a second order. Contact support if it doesn't clear.",
          ],
        },
        {
          heading: "Cash on Delivery (COD)",
          body: [
            "Just select Cash on Delivery at checkout, and pay in cash when your order is delivered — there's no extra fee for choosing COD.",
          ],
        },
        {
          heading: "If your payment doesn't go through",
          body: [
            "You can try again right away from your bag — you're only charged once a payment actually succeeds.",
            "If money was deducted from your bank but your order wasn't confirmed, contact us with your order and payment reference and we'll help sort it out.",
          ],
        },
      ]}
    />
  );
}
