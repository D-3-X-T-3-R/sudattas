import { cache } from "react";
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ProductPageClient } from "@/components/product-page-client";
import type { Product } from "@/lib/schemas";
import type { ProductListRowWithVariantStock } from "@/lib/graphql-types";
import { parsePaise, paiseToRupeesNumber } from "@/lib/money";
import {
  fetchProductByIdWithVariantStock,
  fetchCategoriesWithSession,
  fetchSizesWithSession,
  fetchProductsListWithSession,
} from "@/lib/storefront-queries";
import {
  mintGuestSessionIdSingleFlight,
  withRecoveredGuestSession,
} from "@/lib/server-guest-session";
import { forwardedIpHeadersFromCurrentRequest } from "@/lib/forwarded-ip";
import { siteUrl } from "@/lib/site-url";
import { safeJsonLd } from "@/lib/json-ld";

interface ProductPageData {
  product: Product;
  sizes: { sizeId: string; sizeName: string }[];
  relatedProducts: Product[];
}

function absoluteImageUrl(base: string, image: string | undefined): string {
  if (!image) return `${base}/placeholder.jpg`;
  if (image.startsWith("http://") || image.startsWith("https://")) return image;
  return `${base}${image.startsWith("/") ? "" : "/"}${image}`;
}

function mapToStorefrontProduct(
  row: ProductListRowWithVariantStock,
  categoryNameById: Record<string, string>
): Product {
  const pricePaise = parsePaise(row.amountPaise);
  const priceRupees = paiseToRupeesNumber(pricePaise);
  const priceFormatted = row.formatted?.trim() || undefined;
  const imageList = row.images?.filter(
    (i) => i.url || i.thumbnailUrl
  ) as { url?: string | null; thumbnailUrl?: string | null }[] | undefined;
  const allUrls =
    imageList?.map((i) => i.url || i.thumbnailUrl || "").filter(Boolean) ?? [];
  const imageUrl = allUrls[0] ?? "";
  const hoverUrl = allUrls[1] ?? imageUrl;

  return {
    id: row.productId,
    name: row.name,
    collection:
      (row.categoryId && categoryNameById[row.categoryId]) || "Collection",
    price: priceRupees,
    pricePaise,
    priceFormatted,
    rating: 4.5,
    reviews: 0,
    fabric: row.fabric ?? "",
    occasion: row.occasion ?? "",
    description: row.description ?? "",
    image: imageUrl,
    hoverImage: hoverUrl || undefined,
    images: allUrls.length > 0 ? allUrls : undefined,
    imageAlt: row.name,
    variantStock: row.variantStock ?? undefined,
  };
}

const getProductPageData = cache(async (id: string): Promise<ProductPageData | null> => {
  if (!id) return null;
  const forwardedHeaders = await forwardedIpHeadersFromCurrentRequest();
  const sessionId = await mintGuestSessionIdSingleFlight(forwardedHeaders);
  const recovered = await withRecoveredGuestSession(
    sessionId,
    forwardedHeaders,
    async (activeSessionId) => {
      const [row, categories, sizes] = await Promise.all([
        fetchProductByIdWithVariantStock(activeSessionId, id, forwardedHeaders),
        fetchCategoriesWithSession(activeSessionId, forwardedHeaders),
        fetchSizesWithSession(activeSessionId, forwardedHeaders),
      ]);
      const relatedRows = row?.categoryId
        ? await fetchProductsListWithSession(
            activeSessionId,
            { categoryId: row.categoryId, limit: "8" },
            forwardedHeaders
          )
        : [];
      return { row, categories, sizes, relatedRows };
    }
  );

  const row = recovered.value.row;
  if (!row) return null;
  const categoryNameById: Record<string, string> = {};
  for (const c of recovered.value.categories) {
    categoryNameById[c.categoryId] = c.name;
  }
  const relatedProducts = recovered.value.relatedRows
    .filter((r) => r.productId !== row.productId)
    .slice(0, 6)
    .map((r) => mapToStorefrontProduct(r, categoryNameById));
  return {
    product: mapToStorefrontProduct(row, categoryNameById),
    sizes: recovered.value.sizes,
    relatedProducts,
  };
});

export async function generateMetadata({
  params,
}: {
  params: Promise<{ id: string }>;
}): Promise<Metadata> {
  const { id } = await params;
  const data = await getProductPageData(id);
  if (!data) {
    return {
      title: "Product Not Found | Sudatta's",
      description: "The product you are looking for does not exist.",
    };
  }

  const { product } = data;
  const base = siteUrl();
  const canonical = `${base}/product/${encodeURIComponent(product.id)}`;
  const image = absoluteImageUrl(base, product.image);

  return {
    title: `${product.name} | Sudatta's`,
    description: product.description || `Buy ${product.name} online from Sudatta's.`,
    alternates: { canonical },
    openGraph: {
      title: `${product.name} | Sudatta's`,
      description: product.description || `Buy ${product.name} online from Sudatta's.`,
      type: "website",
      url: canonical,
      images: [{ url: image, alt: product.imageAlt || product.name }],
    },
  };
}

export default async function ProductPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const data = await getProductPageData(id);
  if (!data) {
    notFound();
  }

  const { product, sizes, relatedProducts } = data;
  const base = siteUrl();
  const productUrl = `${base}/product/${encodeURIComponent(product.id)}`;
  const image = absoluteImageUrl(base, product.image);
  const priceValue =
    product.pricePaise != null ? Number(product.pricePaise) / 100 : Number(product.price);
  const productJsonLd = {
    "@context": "https://schema.org",
    "@type": "Product",
    name: product.name,
    description: product.description,
    image: [image],
    sku: product.id,
    category: product.collection,
    brand: {
      "@type": "Brand",
      name: "Sudatta's",
    },
    offers: {
      "@type": "Offer",
      priceCurrency: "INR",
      price: Number(priceValue.toFixed(2)),
      availability: "https://schema.org/InStock",
      url: productUrl,
    },
    aggregateRating:
      product.rating > 0
        ? {
            "@type": "AggregateRating",
            ratingValue: product.rating,
            reviewCount: product.reviews ?? 1,
          }
        : undefined,
  };
  const breadcrumbJsonLd = {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: [
      {
        "@type": "ListItem",
        position: 1,
        name: "Home",
        item: `${base}/`,
      },
      {
        "@type": "ListItem",
        position: 2,
        name: product.collection || "Products",
        item: `${base}/`,
      },
      {
        "@type": "ListItem",
        position: 3,
        name: product.name,
        item: productUrl,
      },
    ],
  };

  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: safeJsonLd(productJsonLd) }}
      />
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: safeJsonLd(breadcrumbJsonLd) }}
      />
      <ProductPageClient product={product} sizes={sizes} relatedProducts={relatedProducts} />
    </>
  );
}


