"use client";

import Image from "next/image";
import Link from "next/link";
import { Kicker } from "@/components/ui/typography";

interface FooterProps {
  goTo?: (id: string, instant?: boolean) => void;
}

export function Footer({ goTo }: FooterProps) {
  return (
    <footer className="mt-16 border-t border-[var(--color-line)] bg-[var(--color-surface-soft)] py-12 md:mt-20 md:py-16">
      <div className="mx-auto grid w-full max-w-[var(--container-max)] gap-10 px-[var(--gutter-mobile)] md:grid-cols-4 md:px-[var(--gutter-tablet)]">
        <div>
          <div className="flex items-center gap-3">
            <span className="h-px w-8 bg-[var(--color-gold)]" />
            <Image
              src="/logo.png"
              alt="Sudatta's"
              width={170}
              height={52}
              className="h-8 w-auto"
            />
          </div>
          <p className="mt-4 text-sm leading-relaxed text-[var(--color-muted)]">
            Handcrafted Indian occasionwear that feels timeless, elegant, and
            effortlessly wearable.
          </p>
        </div>

        <div>
          <Kicker className="text-[var(--color-ink)]">Shop</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li>
              {goTo ? (
                <button
                  type="button"
                  onClick={() => goTo("collections", false)}
                  className="hover:text-[var(--color-green)]"
                >
                  Collections
                </button>
              ) : (
                <Link href="/collections" className="hover:text-[var(--color-green)]">
                  Collections
                </Link>
              )}
            </li>
            <li>
              {goTo ? (
                <button
                  type="button"
                  onClick={() => goTo("shop", false)}
                  className="hover:text-[var(--color-green)]"
                >
                  New Arrivals
                </button>
              ) : (
                <Link href="/#shop" className="hover:text-[var(--color-green)]">
                  New Arrivals
                </Link>
              )}
            </li>
            <li>
              <Link href="/contact-support" className="hover:text-[var(--color-green)]">
                Styling Support
              </Link>
            </li>
            <li>
              <Link href="/about" className="hover:text-[var(--color-green)]">
                About Sudatta&apos;s
              </Link>
            </li>
          </ul>
        </div>

        <div>
          <Kicker className="text-[var(--color-ink)]">Policies</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li><Link href="/shipping-policy" className="hover:text-[var(--color-green)]">Shipping Policy</Link></li>
            <li><Link href="/returns-exchanges" className="hover:text-[var(--color-green)]">Returns & Exchanges</Link></li>
            <li><Link href="/terms-conditions" className="hover:text-[var(--color-green)]">Terms & Conditions</Link></li>
            <li><Link href="/privacy-policy" className="hover:text-[var(--color-green)]">Privacy Policy</Link></li>
            <li><Link href="/cancellation-policy" className="hover:text-[var(--color-green)]">Cancellation Policy</Link></li>
            <li><Link href="/payment-guide" className="hover:text-[var(--color-green)]">COD & Prepaid Guide</Link></li>
            <li><Link href="/size-fit-guide" className="hover:text-[var(--color-green)]">Size & Fit Guide</Link></li>
          </ul>
        </div>

        <div>
          <Kicker className="text-[var(--color-ink)]">Contact</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li>Order support available from your account orders.</li>
            <li>
              <Link href="/contact-support" className="hover:text-[var(--color-green)]">
                Support, styling, and bulk enquiries
              </Link>
            </li>
            <li>Email support@sudattas.com for quick assistance.</li>
          </ul>
          <div className="mt-6 flex items-center gap-2 text-xs text-[var(--color-muted)]">
            <span className="h-px w-5 bg-[var(--color-gold)]" />
            (c) {new Date().getFullYear()} Sudatta&apos;s.
          </div>
        </div>
      </div>
    </footer>
  );
}
