import Link from "next/link";
import { motion } from "framer-motion";
import type { CartLine, Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";
import { formatInrFromPaise } from "@/lib/money";
import { BagSizeDropdown } from "@/domains/bag/components/bag-size-dropdown";
import { CheckIcon, HeartIcon, TrashIcon, buildSizeOptions } from "@/domains/bag/components/bag-shared";

type CatalogSize = { sizeId: string; sizeName: string };

type BagLineCardProps = {
  line: CartLine;
  catalogSizes: CatalogSize[];
  isSelected: boolean;
  isSizeMenuOpen: boolean;
  reduceMotion: boolean;
  onOpenSizeChange: (isOpen: boolean) => void;
  onSelect: () => void;
  onDec: (id: string) => void;
  onInc: (id: string) => void;
  onRemove: (id: string) => Promise<void> | void;
  onMoveToWishlist: (line: CartLine) => Promise<void>;
  onReplaceSize: (line: CartLine, newSize: string) => Promise<void>;
};

export function BagLineCard({
  line,
  catalogSizes,
  isSelected,
  isSizeMenuOpen,
  reduceMotion,
  onOpenSizeChange,
  onSelect,
  onDec,
  onInc,
  onRemove,
  onMoveToWishlist,
  onReplaceSize,
}: BagLineCardProps) {
  const { id, product, qty, sizeName } = line;
  const options = buildSizeOptions(product as Product, catalogSizes);
  const hasCurrent = !!sizeName && options.some((option) => option.sizeName === sizeName);
  const descriptor = [product.collection, product.fabric]
    .map((value) => value?.trim())
    .filter(Boolean)
    .join(" | ");

  return (
    <motion.article
      layout
      initial={reduceMotion ? false : { opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={reduceMotion ? { opacity: 0 } : { opacity: 0, y: 8 }}
      transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
      className={cn(
        "relative rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-2.5 shadow-[var(--shadow-subtle)] md:p-4",
        isSizeMenuOpen ? "z-20 overflow-visible" : "z-0 overflow-hidden"
      )}
    >
      <div className="grid grid-cols-[5.5rem_minmax(0,1fr)] gap-3 md:grid-cols-[140px_minmax(0,1fr)] md:gap-4 md:pr-8">
        <Link
          href={`/product/${product.id}`}
          className="block aspect-[4/5] w-full overflow-hidden rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface-soft)]"
        >
          {product.image ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img src={product.image} alt={product.name} className="h-full w-full object-cover" />
          ) : (
            <div className="aspect-[4/5] w-full" />
          )}
        </Link>

        <div className="min-w-0">
          <div className="md:flex md:items-start md:justify-between md:gap-3">
            <div className="min-w-0">
              <Link href={`/product/${product.id}`}>
                <p className="line-clamp-2 font-display text-[1.08rem] leading-[1.12] text-[var(--color-ink)] md:text-2xl">
                  {product.name}
                </p>
              </Link>
              {descriptor ? (
                <p className="mt-1 line-clamp-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)] md:tracking-[0.18em]">
                  {descriptor}
                </p>
              ) : null}
            </div>
            <p className="mt-2 font-sans text-base font-semibold leading-tight text-[var(--color-ink)] md:mt-0 md:shrink-0 md:text-xl">
              {formatInrFromPaise(qty * (product.pricePaise ?? Math.round(product.price * 100)))}
            </p>
          </div>

          <div className="mt-2 flex flex-wrap items-center gap-2 md:mt-4">
            {options.length > 0 ? (
              <BagSizeDropdown
                options={options}
                sizeName={sizeName}
                hasCurrent={hasCurrent}
                onOpenChange={onOpenSizeChange}
                onSelectSize={(newSize) => onReplaceSize(line, newSize)}
              />
            ) : null}

            <div className="inline-flex h-9 items-center rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface)] p-1">
              <button
                type="button"
                onClick={() => onDec(id)}
                aria-label={`Decrease quantity for ${product.name}`}
                className="h-7 w-7 rounded-sm text-base text-[var(--color-ink)] md:h-8 md:w-8 md:text-lg"
              >
                -
              </button>
              <span className="min-w-[2rem] text-center text-sm font-semibold text-[var(--color-ink)]">{qty}</span>
              <button
                type="button"
                onClick={() => onInc(id)}
                aria-label={`Increase quantity for ${product.name}`}
                className="h-7 w-7 rounded-sm text-base text-[var(--color-ink)] md:h-8 md:w-8 md:text-lg"
              >
                +
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="mt-3 grid grid-cols-2 divide-x divide-[var(--color-line)] border-t border-[var(--color-line)] pt-2 md:ml-[156px] md:flex md:items-center md:gap-4 md:divide-x-0 md:pt-3">
        <button
          type="button"
          onClick={() => onRemove(id)}
          aria-label={`Remove ${product.name} from bag`}
          className="inline-flex min-h-9 items-center justify-center gap-1.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-[#8A5D52] hover:text-[#73493f] md:min-h-0 md:justify-start md:text-[11px]"
        >
          <TrashIcon className="h-3.5 w-3.5 md:h-4 md:w-4" />
          Remove
        </button>
        <button
          type="button"
          onClick={() => onMoveToWishlist(line)}
          aria-label={`Move ${product.name} to wishlist`}
          className="inline-flex min-h-9 items-center justify-center gap-1.5 pl-2 text-[10px] font-semibold uppercase tracking-[0.14em] text-[var(--color-green)] hover:text-[var(--color-green-2)] md:min-h-0 md:justify-start md:pl-0 md:text-[11px]"
        >
          <HeartIcon className="h-3.5 w-3.5 md:h-4 md:w-4" />
          Move To Wishlist
        </button>
      </div>

      <button
        type="button"
        onClick={onSelect}
        aria-label={`Select ${product.name}`}
        className={cn(
          "absolute left-2 top-2 z-10 flex h-6 w-6 items-center justify-center rounded-[var(--radius-sm)] border shadow-[0_2px_8px_rgba(45,42,38,0.08)] md:left-auto md:right-4 md:top-4",
          isSelected
            ? "border-[var(--color-green)] bg-[var(--color-green)] text-white"
            : "border-[var(--color-line-strong)] bg-[var(--color-surface)] text-transparent"
        )}
      >
        <CheckIcon className="h-3.5 w-3.5" />
      </button>
    </motion.article>
  );
}
