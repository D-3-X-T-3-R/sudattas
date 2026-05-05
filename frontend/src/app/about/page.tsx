import { PolicyPageShell } from "@/components/policy-page-shell";

export default function AboutPage() {
  return (
    <PolicyPageShell
      eyebrow="About Sudatta's"
      title="A boutique focused on graceful Indian occasion wear."
      intro="Sudatta's curates sarees and kurtis with a balance of traditional drape and modern styling. Our storefront is designed to keep product, payment, and delivery status transparent at every stage."
      sections={[
        {
          heading: "What we focus on",
          body: [
            "We focus on quality-led selection, clear product presentation, and reliable order updates from checkout to delivery.",
            "Each order timeline is tied to verified payment, shipment milestones, and support-ready status updates so customers can make decisions confidently.",
          ],
        },
        {
          heading: "How we support customers",
          body: [
            "For order, cancellation, refund, and delivery queries, support is anchored to your order details to reduce back-and-forth and avoid ambiguity.",
            "For sizing or styling help before placing an order, our support team can guide fit expectations using available product details.",
          ],
        },
      ]}
    />
  );
}
