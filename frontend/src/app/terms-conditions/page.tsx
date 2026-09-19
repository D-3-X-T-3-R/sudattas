import { PolicyPageShell } from "@/components/policy-page-shell";

export default function TermsAndConditionsPage() {
  return (
    <PolicyPageShell
      eyebrow="Terms & Conditions"
      title="Order, payment, and service terms for Sudatta's."
      intro="By placing an order with us, you're confirming that your delivery details, chosen items, and payment are accurate, and that you're authorized to use the payment method provided."
      sections={[
        {
          heading: "Orders and payment",
          body: [
            "We confirm your order only after your payment is verified, and we set aside the exact items you ordered for you.",
            "We take care to avoid charging you twice, refunding you twice, or losing track of your order status. If something looks off, please contact us.",
          ],
        },
        {
          heading: "Cancellations and returns",
          body: [
            "You can cancel your order within the cancellation window shown on your order page. After that, cancellation may not be possible — see our Cancellation Policy and Returns & Exchanges pages for full details.",
            "In rare cases, a cancellation may take a little time to confirm with our courier partner. We'll keep your order updated throughout.",
          ],
        },
        {
          heading: "Support and resolution",
          body: [
            "For any support request, please have your order number ready. We may ask for photos, courier updates, or payment details to help resolve things quickly.",
          ],
        },
        {
          heading: "Intellectual property",
          body: [
            "Everything on this site — our name, logo, product photography, and descriptions — belongs to Sudatta's (or our licensors) and may not be copied or reused without our permission.",
          ],
        },
        {
          heading: "Limitation of liability",
          body: [
            "We take care to keep this site accurate and running smoothly, but we can't guarantee it will always be error-free or uninterrupted.",
            "To the extent the law allows, we aren't responsible for indirect or incidental losses arising from your use of this site or your order, beyond the value of the order itself.",
          ],
        },
        {
          heading: "Governing law",
          body: [
            "These terms are governed by the laws of India.",
          ],
        },
      ]}
    />
  );
}
