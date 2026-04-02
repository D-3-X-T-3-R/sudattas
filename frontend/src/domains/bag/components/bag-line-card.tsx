import Link from "next/link";
import { motion } from "framer-motion";
import type { CartLine, Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";
import { formatInrFromPaise } from "@/lib/money";
import { BagSizeDropdown } from "@/domains/bag/components/bag-size-dropdown";
import {
  CheckIcon,
  GoldDivider,
  HeartIcon,
  TrashIcon,
  buildSizeOptions,
} from "@/domains/bag/components/bag-shared";

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
    <motion.div
      layout
      initial={reduceMotion ? false : { opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={
        reduceMotion
          ? { opacity: 0, transition: { duration: 0.15 } }
          : {
              opacity: 0,
              x: -48,
              scale: 0.96,
              filter: "blur(3px)",
              transition: { duration: 0.4, ease: [0.22, 1, 0.36, 1] },
            }
      }
      transition={{
        layout: { duration: 0.35, ease: [0.22, 1, 0.36, 1] },
        opacity: { duration: 0.25 },
        y: { duration: 0.3 },
      }}
      className={cn(
        "group relative overflow-hidden rounded-[22px] border border-[#0F3D2E]/10 bg-[linear-gradient(180deg,#FFFDF9_0%,#FAF6EF_100%)] shadow-[0_2px_8px_rgba(15,61,46,0.06)] will-change-transform transition duration-500",
        isSizeMenuOpen ? "z-30" : "z-0"
      )}
    >
      <div className="grid grid-cols-[minmax(0,100px)_minmax(0,1fr)] gap-0 sm:grid-cols-[minmax(0,118px)_minmax(0,1fr)] lg:grid-cols-[188px_minmax(0,1fr)]">
        <div className="relative flex flex-col justify-center border-r border-[#0F3D2E]/6 bg-[radial-gradient(circle_at_top_left,rgba(201,166,70,0.10),transparent_40%),linear-gradient(180deg,#F6F0E7_0%,#EFE6D9_100%)] p-2 sm:p-3 lg:p-4">
          <Link
            href={`/product/${product.id}`}
            className="mx-auto block h-[132px] w-[88px] overflow-hidden rounded-[14px] border border-white/55 bg-white shadow-[0_12px_28px_rgba(15,61,46,0.10)] sm:h-[156px] sm:w-[102px] sm:rounded-[16px] lg:h-[182px] lg:w-[128px] lg:rounded-[18px]"
          >
            {product.image ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={product.image}
                alt={product.name}
                className="h-full w-full object-cover transition duration-700 group-hover:scale-[1.035]"
              />
            ) : (
              <div className="h-full w-full" />
            )}
          </Link>
        </div>

        <div className="min-w-0 p-3 sm:p-4 lg:p-5">
          <div>
            <div className="flex items-start justify-between gap-3">
              <div className="flex-1">
                <Link href={`/product/${product.id}`}>
                  <p className="font-display text-[1.4rem] leading-tight text-[#0F3D2E] sm:text-[1.6rem]">
                    {product.name}
                  </p>
                </Link>
                <p className="mt-1 text-[10px] font-semibold uppercase tracking-[0.3em] text-[#8B816D]">
                  {product.collection}
                </p>
              </div>
              <div className="shrink-0 text-right">
                <p className="font-display text-[1.4rem] leading-none text-[#0F3D2E] sm:text-[1.6rem]">
                  {formatInrFromPaise(qty * (product.pricePaise ?? Math.round(product.price * 100)))}
                </p>
                <p className="mt-1 text-[10px] tracking-[0.04em] text-[#807769]">MRP incl. of all taxes</p>
              </div>
            </div>
            <div className="my-3">
              <GoldDivider />
            </div>
            <div className="flex flex-wrap items-center gap-2">
              {options.length > 0 && (
                <BagSizeDropdown
                  options={options}
                  sizeName={sizeName}
                  hasCurrent={hasCurrent}
                  onOpenChange={onOpenSizeChange}
                  onSelectSize={(newSize) => onReplaceSize(line, newSize)}
                />
              )}
              <div className="inline-flex h-9 items-center rounded-full border border-[#0F3D2E]/10 bg-[#FFFDF8] px-1.5 shadow-[0_8px_16px_rgba(15,61,46,0.05)]">
                <button
                  onClick={() => onDec(id)}
                  aria-label="Decrease"
                  className="flex h-7 w-7 items-center justify-center rounded-full text-[#0F3D2E] transition duration-300 hover:scale-105 hover:bg-[#0F3D2E]/6"
                >
                  -
                </button>
                <span className="min-w-[2rem] text-center text-sm font-semibold text-[#162019]">{qty}</span>
                <button
                  onClick={() => onInc(id)}
                  aria-label="Increase"
                  className="flex h-7 w-7 items-center justify-center rounded-full text-[#0F3D2E] transition duration-300 hover:scale-105 hover:bg-[#0F3D2E]/6"
                >
                  +
                </button>
              </div>
            </div>
          </div>

          <div className="mt-4 flex flex-wrap items-center gap-5 border-t border-[#0F3D2E]/8 pt-4">
            <button
              onClick={() => onRemove(id)}
              className="group inline-flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.22em] text-[#9B6A62] transition duration-300 hover:text-[#7B3F38]"
            >
              <TrashIcon className="h-4 w-4 transition duration-300 group-hover:-translate-y-0.5" />
              Remove
            </button>
            <button
              type="button"
              onClick={() => onMoveToWishlist(line)}
              className="group inline-flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.22em] text-[#7C7367] transition duration-300 hover:text-[#0F3D2E]"
            >
              <HeartIcon className="h-4 w-4 transition duration-300 group-hover:-translate-y-0.5" />
              Move to Wishlist
            </button>
          </div>
        </div>
      </div>
      <button
        type="button"
        onClick={onSelect}
        aria-label={`Select ${product.name}`}
        className={cn(
          "absolute bottom-4 right-4 flex h-7 w-7 items-center justify-center rounded-full border-2 shadow-[0_4px_12px_rgba(15,61,46,0.20)] transition-colors duration-200",
          isSelected
            ? "border-[#0F3D2E] bg-[#0F3D2E] text-white"
            : "border-[#0F3D2E]/20 bg-white/60 text-transparent backdrop-blur-sm"
        )}
      >
        <CheckIcon className="h-3.5 w-3.5" />
      </button>
    </motion.div>
  );
}
