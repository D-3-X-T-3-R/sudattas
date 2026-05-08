"use client";

import { useMemo, useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { SiteHeader } from "@/components/site-header";
import { Footer } from "@/components/footer";
import type { StorefrontCollectionPageData } from "@/lib/storefront-collection-page";
import { PageShell, SectionHeader, EmptyState } from "@/components/ui/page-shell";

const PLACEHOLDER_IMAGE =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='500' viewBox='0 0 400 500'%3E%3Crect fill='%23f0ede8' width='400' height='500'/%3E%3Ctext fill='%23999' font-family='sans-serif' font-size='14' x='50%25' y='50%25' dominant-baseline='middle' text-anchor='middle'%3ENo image%3C/text%3E%3C/svg%3E";

function isExternalProductImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    const host = new URL(src).hostname;
    return host !== "images.unsplash.com";
  } catch {
    return false;
  }
}

function parsePriceToNumber(label: string): number {
  const digits = label.replace(/[^0-9.]/g, "");
  const value = Number.parseFloat(digits);
  return Number.isFinite(value) ? value : 0;
}

export function StorefrontCollectionPageContent({
  data,
}: {
  data: StorefrontCollectionPageData;
}) {
  const [sortBy, setSortBy] = useState("Featured");

  const sortedProducts = useMemo(() => {
    const list = [...data.products];
    if (sortBy === "Price: Low") {
      list.sort((a, b) => parsePriceToNumber(a.priceLabel) - parsePriceToNumber(b.priceLabel));
      return list;
    }
    if (sortBy === "Price: High") {
      list.sort((a, b) => parsePriceToNumber(b.priceLabel) - parsePriceToNumber(a.priceLabel));
      return list;
    }
    if (sortBy === "Name") {
      list.sort((a, b) => a.name.localeCompare(b.name));
      return list;
    }
    return list;
  }, [data.products, sortBy]);

  const filterPanel = (
    <div className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)]">
      <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[var(--color-muted)]">Filter By</p>
      <div className="mt-4 space-y-4">
        <div>
          <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">Category</p>
          <p className="mt-1 text-sm text-[var(--color-ink)]">{data.categoryName}</p>
        </div>
        <div>
          <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">Price</p>
          <p className="mt-1 text-sm text-[var(--color-ink)]">Curated premium range</p>
        </div>
        <div>
          <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">Availability</p>
          <p className="mt-1 text-sm text-[var(--color-ink)]">{data.products.length} styles available</p>
        </div>
      </div>
    </div>
  );

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      <SiteHeader />
      <PageShell wide containerClassName="py-8 md:py-10">
        <SectionHeader
          label="Collection"
          title={data.categoryName}
          description="Thoughtfully designed pieces with premium fabrics and timeless silhouettes."
          action={
            <div className="flex items-center gap-3">
              <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">
                {sortedProducts.length} items
              </span>
              <label className="text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">
                Sort
                <select
                  value={sortBy}
                  onChange={(e) => setSortBy(e.target.value)}
                  className="ml-2 h-10 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 text-sm text-[var(--color-ink)]"
                >
                  <option>Featured</option>
                  <option>Price: Low</option>
                  <option>Price: High</option>
                  <option>Name</option>
                </select>
              </label>
            </div>
          }
        />

        {sortedProducts.length === 0 ? (
          <div className="mt-8">
            <EmptyState
              title="No products available"
              description="This collection is currently being refreshed. Please check back shortly."
            />
          </div>
        ) : (
          <>
            <div className="mt-6 md:hidden">
              <details className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)]">
                <summary className="cursor-pointer text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)]">Filters</summary>
                <div className="mt-4">{filterPanel}</div>
              </details>
            </div>

            <section className="mt-8 grid items-start gap-6 md:grid-cols-[260px_minmax(0,1fr)]">
              <aside className="hidden md:block">{filterPanel}</aside>
              <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4 md:gap-5">
                {sortedProducts.map((product) => (
                  <article key={product.id} className="group rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] shadow-[var(--shadow-subtle)]">
                    <Link
                      href={`/product/${encodeURIComponent(product.id)}`}
                      className="block overflow-hidden rounded-t-lg border-b border-[var(--color-line)]"
                    >
                      <div className="relative aspect-[3/4] w-full">
                        <Image
                          src={product.imageUrl || PLACEHOLDER_IMAGE}
                          alt={product.name}
                          fill
                          className="object-cover transition-transform duration-500 group-hover:scale-[1.03]"
                          sizes="(max-width: 768px) 50vw, 25vw"
                          unoptimized={isExternalProductImage(product.imageUrl)}
                        />
                      </div>
                    </Link>
                    <div className="p-3 sm:p-4">
                      <h2 className="line-clamp-2 font-display text-lg leading-tight text-[var(--color-ink)] sm:text-[1.35rem]">
                        {product.name}
                      </h2>
                      <p className="mt-2 font-sans text-lg font-semibold text-[var(--color-ink)]">{product.priceLabel}</p>
                      <p className="mt-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--color-gold)]">New</p>
                    </div>
                  </article>
                ))}
              </div>
            </section>
          </>
        )}
      </PageShell>
      <Footer />
    </div>
  );
}
