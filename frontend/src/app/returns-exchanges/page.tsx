import { PolicyPageShell } from "@/components/policy-page-shell";

export default function ReturnsAndExchangesPage() {
  return (
    <PolicyPageShell
      eyebrow="Returns & Exchanges"
      title="Returns are reviewed with care, not left to guesswork."
      intro="If you receive the wrong item, a damaged parcel, or an order that is materially different from what was confirmed, reach out through support from your account order view and we will guide the next steps."
      sections={[
        {
          heading: "Before pickup completion",
          body: [
            "Customer cancellation remains available until pickup is completed by the logistics partner. During this window, our system cancels the shipment first, then restores stock and initiates the refund.",
            "Once pickup is completed, customer self-cancel is disabled and any refusal or return is handled through the courier return-to-origin workflow.",
          ],
        },
        {
          heading: "Return review",
          body: [
            "Approved return or refused-delivery outcomes are reconciled against the shipment status so inventory and refunds are only processed once.",
            "Refund status and any courier return milestone will appear on your order detail timeline as they are confirmed.",
          ],
        },
        {
          heading: "Exchange support",
          body: [
            "For sizing, styling, or order support, please contact us before dispatch where possible. We will advise whether a cancellation-and-reorder or a post-delivery support review is the best path.",
          ],
        },
      ]}
    />
  );
}
