import Image from "next/image";
import Link from "next/link";
import type { StorefrontCollectionPageData } from "@/lib/storefront-collection-page";

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

export function StorefrontCollectionPageContent({
  data,
}: {
  data: StorefrontCollectionPageData;
}) {
  return (
    <main className="mx-auto w-full max-w-[1200px] px-4 py-10 sm:px-6 lg:px-8">
      <div className="border-b border-[var(--color-line)] pb-6">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
          Collection
        </p>
        <h1 className="mt-2 font-display text-3xl tracking-tight text-[var(--color-ink)] sm:text-4xl">
          {data.categoryName}
        </h1>
        <p className="mt-2 text-sm text-[var(--color-muted)]">
          {data.products.length} product{data.products.length === 1 ? "" : "s"}
        </p>
      </div>

      {data.products.length === 0 ? (
        <section className="rounded-sm border border-[var(--color-line)] bg-[var(--background)] px-6 py-10 text-center text-[var(--color-muted)]">
          No products are available in this collection right now.
        </section>
      ) : (
        <section className="mt-8 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
          {data.products.map((product) => (
            <article key={product.id} className="group">
              <Link
                href={`/product/${encodeURIComponent(product.id)}`}
                className="block overflow-hidden rounded-sm border border-[var(--color-line)] bg-white transition hover:shadow-[0_8px_24px_rgba(26,24,20,0.08)]"
              >
                <div className="relative aspect-[4/5] w-full">
                  <Image
                    src={product.imageUrl || PLACEHOLDER_IMAGE}
                    alt={product.name}
                    fill
                    className="object-cover transition-transform duration-300 group-hover:scale-[1.03]"
                    sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 25vw"
                    unoptimized={isExternalProductImage(product.imageUrl)}
                  />
                </div>
                <div className="p-4">
                  <h2 className="line-clamp-2 font-display text-lg tracking-tight text-[var(--color-ink)]">
                    {product.name}
                  </h2>
                  <p className="mt-1 text-xs uppercase tracking-[0.18em] text-[var(--color-muted)]">
                    {product.categoryName}
                  </p>
                  <p className="mt-2 font-semibold text-[var(--color-accent-gold)]">
                    {product.priceLabel}
                  </p>
                </div>
              </Link>
            </article>
          ))}
        </section>
      )}
    </main>
  );
}
