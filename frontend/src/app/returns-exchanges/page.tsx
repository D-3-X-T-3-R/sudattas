import { PolicyPageShell } from "@/components/policy-page-shell";

export default function ReturnsAndExchangesPage() {
  return (
    <PolicyPageShell
      eyebrow="Returns & Exchanges"
      title="Wrong size? You can exchange it yourself. Anything else, we'll sort out personally."
      intro="Need a different size? You can request an exchange yourself, right from your order — no need to contact us first. We don't offer refunds, except in rare cases like a damaged product or if we sent you the wrong item. If that happens, just message us from your order page and we'll take care of it."
      sections={[
        {
          heading: "Cancelling your order",
          body: [
            "You can cancel any time before the courier picks up your order, and we'll start your refund right away.",
            "Once it's been picked up, you can no longer cancel it yourself — but you can still refuse it at the door if you'd rather send it back.",
          ],
        },
        {
          heading: "Exchanging for a different size",
          body: [
            "If an item is eligible for exchange, you'll see a \"Request exchange\" button on that order. You can use it within 7 days of delivery to swap for a different size of the same item — completely free. Only one exchange request is allowed per item.",
            "The item should be unused, unworn, and still have its original tags and packaging — basically, in the same condition it arrived in. Exchanges are only available on certain products, and only if the size you want is in stock. If you don't see the \"Request exchange\" button on an order, that item either isn't eligible, or the 7-day window has passed.",
            "Once your request is approved, a courier will pick up the item from you, and we'll send your replacement as soon as we receive it back. You can follow every step right on your order page.",
          ],
        },
        {
          heading: "Refund policy",
          body: [
            "We don't offer refunds as a general rule. The only exceptions are if your product arrives damaged, or if we send you the wrong item.",
            "If you just need a different size, use Request exchange instead — it's quicker. For a damaged or wrong item, message us from your order page and we'll look into it.",
          ],
        },
        {
          heading: "If you refuse delivery, or a parcel can't be delivered",
          body: [
            "If you refuse a delivery, or the courier is unable to deliver your order, your refund will be processed once it's back with us.",
            "You can track the courier's return progress and your refund status right on your order page.",
          ],
        },
      ]}
    />
  );
}
