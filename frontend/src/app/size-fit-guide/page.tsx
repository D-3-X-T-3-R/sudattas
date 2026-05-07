import { PolicyPageShell } from "@/components/policy-page-shell";

export default function SizeFitGuidePage() {
  return (
    <PolicyPageShell
      eyebrow="Size & Fit Guide"
      title="Fit guidance for sarees and kurtis before you order."
      intro="Fabric, drape style, and cut can change how a product feels on the body. Use this guide as directional help and contact us if you are unsure between options."
      sections={[
        {
          heading: "Saree fit guidance",
          body: [
            "Most sarees are drape-driven, and the final look depends on blouse fit, pleat style, and pallu drape.",
            "Check product fabric details and fall behavior. If you need help deciding for an event, contact support before ordering.",
          ],
        },
        {
          heading: "Kurti fit guidance",
          body: [
            "Kurti fit can vary by cut and fabric structure, even within the same nominal size.",
            "Review available size options and choose based on your comfortable bust/waist/hip preferences rather than label familiarity alone.",
          ],
        },
        {
          heading: "When size options are limited",
          body: [
            "Some products may have limited or free-size style availability depending on stock and design.",
            "If you are unsure whether a style suits your fit expectations, contact us before placing the order.",
          ],
        },
      ]}
    />
  );
}
