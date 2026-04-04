import "server-only";

import { forwardedIpHeadersFromCurrentRequest } from "@/lib/forwarded-ip";
import { parsePaise, paiseToRupeesNumber } from "@/lib/money";
import {
  fetchCategoriesWithSession,
  fetchProductsListWithSession,
} from "@/lib/storefront-queries";
import {
  mintGuestSessionIdSingleFlight,
  withRecoveredGuestSession,
} from "@/lib/server-guest-session";

export interface CollectionCardProduct {
  id: string;
  name: string;
  priceLabel: string;
  imageUrl: string;
  categoryName: string;
}

export interface StorefrontCollectionPageData {
  categoryId: string;
  categoryName: string;
  categorySlug: string;
  products: CollectionCardProduct[];
}

function slugifyCategoryName(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function firstImageUrl(
  images:
    | {
        url?: string | null;
        thumbnailUrl?: string | null;
      }[]
    | null
    | undefined
): string {
  for (const image of images ?? []) {
    if (image.url?.trim()) return image.url.trim();
    if (image.thumbnailUrl?.trim()) return image.thumbnailUrl.trim();
  }
  return "";
}

function mapPriceLabel(amountPaise: string | undefined, formatted: string | undefined): string {
  const cleanFormatted = formatted?.trim();
  if (cleanFormatted) return cleanFormatted;
  const valuePaise = parsePaise(amountPaise);
  const inr = paiseToRupeesNumber(valuePaise);
  return new Intl.NumberFormat("en-IN", {
    style: "currency",
    currency: "INR",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(inr);
}

async function loadCollectionByCategoryId(categoryId: string): Promise<StorefrontCollectionPageData | null> {
  const forwardedHeaders = await forwardedIpHeadersFromCurrentRequest();
  const sessionId = await mintGuestSessionIdSingleFlight(forwardedHeaders);
  const recovered = await withRecoveredGuestSession(
    sessionId,
    forwardedHeaders,
    async (activeSessionId) => {
      const [categories, products] = await Promise.all([
        fetchCategoriesWithSession(activeSessionId, forwardedHeaders),
        fetchProductsListWithSession(
          activeSessionId,
          { categoryId: categoryId.trim(), limit: "200" },
          forwardedHeaders
        ),
      ]);
      return { categories, products };
    }
  );

  const category = recovered.value.categories.find((entry) => entry.categoryId === categoryId.trim());
  if (!category) return null;

  return {
    categoryId: category.categoryId,
    categoryName: category.name,
    categorySlug: slugifyCategoryName(category.name),
    products: recovered.value.products.map((row) => ({
      id: row.productId,
      name: row.name,
      priceLabel: mapPriceLabel(row.amountPaise, row.formatted),
      imageUrl: firstImageUrl(row.images),
      categoryName: category.name,
    })),
  };
}

export async function loadCollectionByCategorySlug(
  slug: string
): Promise<StorefrontCollectionPageData | null> {
  const normalizedSlug = slugifyCategoryName(slug);
  if (!normalizedSlug) return null;

  const forwardedHeaders = await forwardedIpHeadersFromCurrentRequest();
  const sessionId = await mintGuestSessionIdSingleFlight(forwardedHeaders);
  const recovered = await withRecoveredGuestSession(
    sessionId,
    forwardedHeaders,
    async (activeSessionId) =>
      fetchCategoriesWithSession(activeSessionId, forwardedHeaders)
  );
  const category = recovered.value.find(
    (entry) => slugifyCategoryName(entry.name) === normalizedSlug
  );
  if (!category) return null;
  return loadCollectionByCategoryId(category.categoryId);
}

export async function loadCollectionByCategoryIdRoute(
  categoryId: string
): Promise<StorefrontCollectionPageData | null> {
  if (!categoryId.trim()) return null;
  return loadCollectionByCategoryId(categoryId);
}

export { slugifyCategoryName };
