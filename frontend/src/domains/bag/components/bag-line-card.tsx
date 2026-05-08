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

  return (
    <motion.article
      layout
      initial={reduceMotion ? false : { opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={reduceMotion ? { opacity: 0 } : { opacity: 0, y: 8 }}
      transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
      className={cn(
        "relative overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-3 shadow-[var(--shadow-subtle)] md:p-4",
        isSizeMenuOpen ? "z-20" : "z-0"
      )}
    >
      <div className="grid gap-3 sm:grid-cols-[120px_minmax(0,1fr)] md:grid-cols-[140px_minmax(0,1fr)]">
        <Link href={`/product/${product.id}`} className="block overflow-hidden rounded-sm border border-[var(--color-line)] bg-white">
          {product.image ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img src={product.image} alt={product.name} className="h-full w-full object-cover" />
          ) : (
            <div className="aspect-[3/4] w-full" />
          )}
        </Link>

        <div className="min-w-0">
          <div className="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
            <div className="min-w-0">
              <Link href={`/product/${product.id}`}>
                <p className="line-clamp-2 font-display text-xl leading-tight text-[var(--color-ink)] md:text-2xl">{product.name}</p>
              </Link>
              <p className="mt-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]">
                {product.collection}
              </p>
            </div>
            <p className="font-sans text-xl font-semibold text-[var(--color-ink)]">
              {formatInrFromPaise(qty * (product.pricePaise ?? Math.round(product.price * 100)))}
            </p>
          </div>

          <div className="mt-4 flex flex-wrap items-center gap-2">
            {options.length > 0 ? (
              <BagSizeDropdown
                options={options}
                sizeName={sizeName}
                hasCurrent={hasCurrent}
                onOpenChange={onOpenSizeChange}
                onSelectSize={(newSize) => onReplaceSize(line, newSize)}
              />
            ) : null}

            <div className="inline-flex items-center rounded-md border border-[var(--color-line)] bg-white p-1">
              <button
                type="button"
                onClick={() => onDec(id)}
                aria-label={`Decrease quantity for ${product.name}`}
                className="h-8 w-8 rounded-sm text-lg text-[var(--color-ink)]"
              >
                -
              </button>
              <span className="min-w-[2rem] text-center text-sm font-semibold text-[var(--color-ink)]">{qty}</span>
              <button
                type="button"
                onClick={() => onInc(id)}
                aria-label={`Increase quantity for ${product.name}`}
                className="h-8 w-8 rounded-sm text-lg text-[var(--color-ink)]"
              >
                +
              </button>
            </div>
          </div>

          <div className="mt-4 flex flex-wrap items-center gap-4 border-t border-[var(--color-line)] pt-3">
            <button
              type="button"
              onClick={() => onRemove(id)}
              aria-label={`Remove ${product.name} from bag`}
              className="inline-flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.14em] text-[#8A5D52] hover:text-[#73493f]"
            >
              <TrashIcon className="h-4 w-4" />
              Remove
            </button>
            <button
              type="button"
              onClick={() => onMoveToWishlist(line)}
              aria-label={`Move ${product.name} to wishlist`}
              className="inline-flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--color-green)] hover:text-[var(--color-green-2)]"
            >
              <HeartIcon className="h-4 w-4" />
              Move To Wishlist
            </button>
          </div>
        </div>
      </div>

      <button
        type="button"
        onClick={onSelect}
        aria-label={`Select ${product.name}`}
        className={cn(
          "absolute right-3 top-3 flex h-6 w-6 items-center justify-center rounded-sm border",
          isSelected
            ? "border-[var(--color-green)] bg-[var(--color-green)] text-white"
            : "border-[var(--color-line-strong)] bg-white text-transparent"
        )}
      >
        <CheckIcon className="h-3.5 w-3.5" />
      </button>
    </motion.article>
  );
}
