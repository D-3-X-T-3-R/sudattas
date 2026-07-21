import type { Metadata } from "next";
import { StorefrontExploreListingPage } from "@/components/storefront-explore-listing-page";

export const metadata: Metadata = {
  title: "Explore | Sudatta's",
  description: "Browse curated sarees, kurtis, and occasion-ready pieces from Sudatta's.",
};

export const dynamic = "force-dynamic";

export default function CollectionsIndexPage() {
  return <StorefrontExploreListingPage />;
}
