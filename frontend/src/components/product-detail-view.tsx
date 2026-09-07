"use client";

import { useMemo, useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { ShieldCheck, Truck, Undo2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Accordion } from "@/components/ui/accordion";
import { ScrollCarousel } from "@/components/ui/carousel";
import { Kicker, SectionHeading } from "@/components/ui/typography";
import { ProductCard } from "@/components/product-card";
// Reviews/ratings are disabled in the frontend for now (backend + component kept intact —
// see product-rating-widget.tsx — re-enable by uncommenting this import and its usage below).
// import { ProductRatingWidget } from "@/components/product-rating-widget";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";

const FALLBACK_SIZE_NAMES = ["Free Size", "XS", "S", "M", "L", "XL", "XXL", "3XL"];

const TRUST_POINTS = [
  { icon: ShieldCheck, text: "Secure checkout and protected payments." },
  { icon: Truck, text: "Delivery timelines confirmed at checkout for your pincode." },
  { icon: Undo2, text: "Easy return support available from your order dashboard." },
] as const;

type VariantStockRow = { sizeName: string; quantity: number };

function hasSizeSelector(product: Product): boolean {
  const stock = product.variantStock;
  if (!Array.isArray(stock) || stock.length === 0) return false;
  return !(stock.length === 1 && stock[0].sizeName.trim().toLowerCase() === "free size");
}

function getStockForSize(variantStock: VariantStockRow[], sizeName: string): number {
  return (
    variantStock.find((s) => s.sizeName.toLowerCase() === sizeName.toLowerCase())
      ?.quantity ?? 0
  );
}

function isExternalImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    return new URL(src).hostname !== "images.unsplash.com";
  } catch {
    return false;
  }
}

function ProductGallery({
  images,
  mainImage,
  productName,
  selectedImageIndex,
  setSelectedImageIndex,
}: {
  images: string[];
  mainImage: string;
  productName: string;
  selectedImageIndex: number;
  setSelectedImageIndex: (value: number) => void;
}) {
  return (
    <div className="grid grid-cols-1 gap-3 md:sticky md:top-24 md:grid-cols-[88px_minmax(0,1fr)] md:gap-4">
      <div className="order-2 flex gap-2.5 overflow-x-auto pb-1 md:order-1 md:flex-col md:overflow-visible">
        {images.map((src, i) => (
          <button
            key={i}
            type="button"
            onClick={() => setSelectedImageIndex(i)}
            aria-label={`View image ${i + 1} of ${images.length} for ${productName}`}
            className={cn(
              "relative aspect-[2/3] w-16 shrink-0 overflow-hidden rounded-[var(--radius-md)] border bg-[var(--color-surface-soft)] md:w-full",
              selectedImageIndex === i
                ? "border-[var(--color-green)]"
                : "border-[var(--color-line)] hover:border-[var(--color-gold)]"
            )}
          >
            <Image
              src={src}
              alt={`${productName} view ${i + 1}`}
              fill
              className="object-cover"
              sizes="88px"
              unoptimized={isExternalImage(src)}
            />
          </button>
        ))}
      </div>

      <div className="order-1 relative aspect-[2/3] overflow-hidden rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface-soft)] shadow-[var(--shadow-soft)] md:order-2">
        <Image
          src={mainImage}
          alt={productName}
          fill
          priority
          className="object-cover"
          sizes="(max-width: 768px) 100vw, 50vw"
          unoptimized={isExternalImage(mainImage)}
        />
      </div>
    </div>
  );
}

function ProductDetails({ product }: { product: Product }) {
  return (
    <div>
      <Accordion title="Product Details" defaultOpen>
        <div className="grid gap-2">
          <div className="flex justify-between gap-2">
            <span>Material</span>
            <span className="text-[var(--color-ink)]">{product.fabric || "N/A"}</span>
          </div>
          <div className="flex justify-between gap-2">
            <span>Occasion</span>
            <span className="text-[var(--color-ink)]">{product.occasion || "N/A"}</span>
          </div>
        </div>
        {product.description ? <p className="mt-3 leading-relaxed">{product.description}</p> : null}
      </Accordion>

      <Accordion title="Delivery & Returns">
        <p className="leading-relaxed">
          Delivery timelines are confirmed at checkout for your pincode. Read our{" "}
          <Link href="/shipping-policy" className="font-semibold text-[var(--color-ink)] underline">
            Shipping Policy
          </Link>{" "}
          and{" "}
          <Link href="/returns-exchanges" className="font-semibold text-[var(--color-ink)] underline">
            Returns &amp; Exchanges
          </Link>{" "}
          for eligibility and process details.
        </p>
      </Accordion>

      <Accordion title="Care Instructions">
        <ul className="list-disc space-y-1 pl-4">
          <li>Dry clean or gentle hand wash recommended.</li>
          <li>Do not bleach or wring aggressively.</li>
          <li>Dry in shade and iron on reverse side.</li>
          <li>Store folded in a breathable cotton cover.</li>
        </ul>
      </Accordion>

      <Accordion title="Need Help?">
        <p className="leading-relaxed">
          Have a question about this piece? Our{" "}
          <Link href="/contact-support" className="font-semibold text-[var(--color-ink)] underline">
            Customer Support
          </Link>{" "}
          team is happy to help.
        </p>
      </Accordion>
    </div>
  );
}

export interface ProductDetailViewProps {
  product: Product;
  sizes?: { sizeId: string; sizeName: string }[];
  wished: boolean;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product, qty?: number, sizeName?: string | null) => void;
  relatedProducts?: Product[];
  relatedWishlist?: Record<string, boolean>;
}

export function ProductDetailView({
  product,
  sizes: sizesFromApi = [],
  wished,
  onToggleWish,
  onAddToCart,
  relatedProducts = [],
  relatedWishlist = {},
}: ProductDetailViewProps) {
  const router = useRouter();
  const [selectedImageIndex, setSelectedImageIndex] = useState(0);
  const variantStock = product.variantStock ?? [];

  const allSizeNames =
    sizesFromApi.length > 0
      ? sizesFromApi.map((s) => s.sizeName)
      : FALLBACK_SIZE_NAMES;

  const sizeNames = hasSizeSelector(product)
    ? allSizeNames.filter((n) => n.trim().toLowerCase() !== "free size")
    : allSizeNames;

  const defaultSize =
    sizeNames.find((name) => getStockForSize(variantStock, name) > 0) ??
    sizeNames[0] ??
    null;

  const [selectedSize, setSelectedSize] = useState<string | null>(defaultSize);
  const [quantity, setQuantity] = useState(1);

  const maxQuantity =
    selectedSize != null
      ? getStockForSize(variantStock, selectedSize)
      : variantStock[0]?.quantity ?? 999;

  const safeQuantity = maxQuantity < 1 ? 1 : Math.min(Math.max(1, quantity), maxQuantity);

  const images = useMemo(() => {
    if (product.images?.length) return product.images;
    const list = [product.image];
    if (product.hoverImage && product.hoverImage !== product.image) {
      list.push(product.hoverImage);
    }
    return list.filter(Boolean).length ? list : [product.image || ""];
  }, [product]);

  const mainImage = images[selectedImageIndex] || product.image;

  return (
    <>
    <section className="grid gap-8 md:grid-cols-[3fr_2fr] md:items-start md:gap-10 lg:gap-14">
      <ProductGallery
        images={images}
        mainImage={mainImage}
        productName={product.imageAlt || product.name}
        selectedImageIndex={selectedImageIndex}
        setSelectedImageIndex={setSelectedImageIndex}
      />

      <div className="md:sticky md:top-24">
        <div className="border-b border-[var(--color-line)] pb-6">
          <Kicker tone="accent">{product.collection}</Kicker>
          <h1 className="mt-2 font-display text-[2.1rem] font-medium leading-[1.12] tracking-[-0.01em] text-[var(--color-ink)] sm:text-[2.5rem]">
            {product.name}
          </h1>
          <p className="mt-3 text-sm leading-relaxed text-[var(--color-muted)]">{product.description}</p>
          <p className="mt-4 font-sans text-2xl font-semibold text-[var(--color-ink)] md:text-[1.75rem]">
            {product.priceFormatted ?? INR.format(product.price)}
          </p>
          {/* Reviews/ratings disabled in the frontend for now — see the import comment above. */}
          {/* <div className="mt-4">
            <ProductRatingWidget
              productId={product.id}
              initialAverage={product.rating}
              initialCount={product.reviews ?? 0}
            />
          </div> */}
        </div>

        <div className="space-y-6 border-b border-[var(--color-line)] py-6">
          {hasSizeSelector(product) ? (
            <div>
              <div className="flex items-center justify-between gap-2">
                <p className="text-sm font-semibold text-[var(--color-ink)]">Select size</p>
                <Link href="/size-fit-guide" className="text-xs font-semibold uppercase tracking-[0.12em] text-[var(--color-green)] hover:text-[var(--color-green-2)]">
                  Size &amp; Fit Guide
                </Link>
              </div>
              <div className="mt-3 flex flex-wrap gap-2">
                {sizeNames.map((sizeName) => {
                  const qty = getStockForSize(variantStock, sizeName);
                  const outOfStock = qty <= 0;
                  return (
                    <button
                      key={sizeName}
                      type="button"
                      disabled={outOfStock}
                      onClick={() => !outOfStock && setSelectedSize(sizeName)}
                      className={cn(
                        "min-w-[3rem] rounded-[var(--radius-md)] border px-3.5 py-2 text-sm font-medium transition-colors",
                        outOfStock && "cursor-not-allowed border-[var(--color-line)] bg-[var(--color-surface-soft)] text-[var(--color-muted)] line-through",
                        !outOfStock && selectedSize === sizeName && "border-[var(--color-green)] bg-[var(--color-green)] text-white",
                        !outOfStock && selectedSize !== sizeName && "border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink)] hover:border-[var(--color-gold)]"
                      )}
                    >
                      {sizeName}
                    </button>
                  );
                })}
              </div>
            </div>
          ) : null}

          <div>
            <p className="text-sm font-semibold text-[var(--color-ink)]">Quantity</p>
            <div className="mt-3 inline-flex items-center rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface)] p-1">
              <button
                type="button"
                onClick={() => setQuantity((q) => Math.max(1, q - 1))}
                disabled={safeQuantity <= 1}
                className="h-8 w-8 rounded-[var(--radius-sm)] text-lg text-[var(--color-ink)] transition-colors hover:bg-[var(--color-surface-soft)] disabled:opacity-40 disabled:hover:bg-transparent"
                aria-label="Decrease quantity"
              >
                -
              </button>
              <span className="min-w-[2.2rem] text-center text-sm font-semibold text-[var(--color-ink)]">{safeQuantity}</span>
              <button
                type="button"
                onClick={() => setQuantity((q) => Math.min(maxQuantity, q + 1))}
                disabled={safeQuantity >= maxQuantity}
                className="h-8 w-8 rounded-[var(--radius-sm)] text-lg text-[var(--color-ink)] transition-colors hover:bg-[var(--color-surface-soft)] disabled:opacity-40 disabled:hover:bg-transparent"
                aria-label="Increase quantity"
              >
                +
              </button>
            </div>
          </div>
        </div>

        <div className="grid gap-2.5 pt-6">
          <Button
            onClick={() => onAddToCart(product, safeQuantity, selectedSize)}
            disabled={maxQuantity < 1}
            className="w-full"
          >
            ADD TO BAG
          </Button>
          <Button variant="outline" onClick={() => onToggleWish(product)} className="w-full">
            {wished ? "Wishlisted" : "Add to Wishlist"}
          </Button>
        </div>

        <ul className="mt-6 grid gap-3">
          {TRUST_POINTS.map(({ icon: Icon, text }) => (
            <li key={text} className="flex items-start gap-2.5 text-xs leading-relaxed text-[var(--color-muted)]">
              <Icon className="mt-0.5 h-4 w-4 shrink-0 text-[var(--color-gold)]" strokeWidth={1.75} />
              <span>{text}</span>
            </li>
          ))}
        </ul>

        <div className="mt-8">
          <ProductDetails product={product} />
        </div>
      </div>
    </section>

    {relatedProducts.length > 0 ? (
      <div className="mt-16 border-t border-[var(--color-line)] pt-12 md:mt-24 md:pt-16">
        <Kicker tone="accent">Complete the Look</Kicker>
        <SectionHeading size="lg" className="mt-2">
          Pair It With
        </SectionHeading>
        <ScrollCarousel className="mt-8">
          {relatedProducts.map((related) => (
            <ProductCard
              key={related.id}
              product={related}
              wished={!!relatedWishlist[related.id]}
              onToggleWish={onToggleWish}
              onQuickView={(p) => router.push(`/product/${p.id}`)}
            />
          ))}
        </ScrollCarousel>
      </div>
    ) : null}
    </>
  );
}
