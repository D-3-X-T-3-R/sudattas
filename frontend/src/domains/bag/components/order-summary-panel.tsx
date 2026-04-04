import { INR } from "@/lib/constants";
import { formatInrFromPaise } from "@/lib/money";
import type { CartLine } from "@/lib/schemas";
import { ArrowRightIcon, BagIcon } from "@/domains/bag/components/bag-shared";

type OrderSummaryPanelProps = {
  cartLines: CartLine[];
  cartSubtotal: number;
  cartCount: number;
  onCheckout: () => void;
};

export function OrderSummaryPanel({
  cartLines,
  cartSubtotal,
  cartCount,
  onCheckout,
}: OrderSummaryPanelProps) {
  return (
    <div className="overflow-hidden rounded-[30px] bg-[radial-gradient(circle_at_top,rgba(201,166,70,0.22),transparent_42%),linear-gradient(165deg,#0E3D2F_0%,#114636_48%,#082E24_100%)] text-[#F6F3EA] shadow-[0_2px_8px_rgba(15,61,46,0.06)]">
      <div className="p-6 sm:p-7">
        <div className="flex items-center gap-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-full border border-white/10 bg-white/10 text-[#E7CF82] shadow-[0_12px_24px_rgba(0,0,0,0.18)]">
            <BagIcon className="h-5 w-5" />
          </div>
          <div>
            <p className="text-[11px] font-semibold uppercase tracking-[0.28em] text-[#E7CF82]">
              Price Details
            </p>
            <p className="mt-1 text-sm text-[#F6F3EA]/78">
              {cartCount} {cartCount === 1 ? "item" : "items"} in your bag
            </p>
          </div>
        </div>

        <div className="mt-7 rounded-[24px] border border-white/10 bg-white/5 p-5 backdrop-blur-sm">
          {cartLines.map(({ id, product, qty, sizeName }) => (
            <div key={id} className="mb-3 flex items-center justify-between gap-4">
              <span className="text-sm text-[#F6F3EA]/82">
                {product.name}
                {sizeName && sizeName.toLowerCase() !== "free size" ? ` / ${sizeName}` : ""} x {qty}
              </span>
              <span className="text-sm font-medium text-white">
                {formatInrFromPaise(qty * (product.pricePaise ?? Math.round(product.price * 100)))}
              </span>
            </div>
          ))}
          <div className="my-4 border-t border-white/10" />
          <div className="flex items-center justify-between gap-4">
            <span className="text-sm text-[#F6F3EA]/82">Subtotal</span>
            <span className="text-sm font-medium text-white">{INR.format(cartSubtotal)}</span>
          </div>
          <div className="mt-3 flex items-center justify-between gap-4">
            <span className="text-sm text-[#F6F3EA]/65">Shipping</span>
            <span className="text-sm font-medium text-[#F6F3EA]/70">Calculated at checkout</span>
          </div>
          <div className="my-5 border-t border-white/10" />
          <div className="flex items-center justify-between gap-4">
            <span className="text-sm font-semibold text-[#F6F3EA]">Total Amount</span>
            <span className="text-3xl font-semibold text-white">{INR.format(cartSubtotal)}</span>
          </div>
        </div>

        <div className="mt-7">
          <button
            onClick={onCheckout}
            className="group inline-flex h-14 w-full items-center justify-center gap-2 rounded-full bg-[#C9A646] px-5 text-[11px] font-semibold uppercase tracking-[0.24em] text-white shadow-[0_16px_28px_rgba(201,166,70,0.28)] transition duration-300 hover:-translate-y-1 hover:bg-[#B89435]"
          >
            Checkout
            <ArrowRightIcon className="h-4 w-4 transition duration-300 group-hover:translate-x-1" />
          </button>
        </div>
      </div>
    </div>
  );
}
