"use client";

import Image from "next/image";
import Link from "next/link";
import { Kicker } from "@/components/ui/typography";

interface FooterProps {
  goTo?: (id: string, instant?: boolean) => void;
}

export function Footer({ goTo }: FooterProps) {
  return (
    <footer className="bg-deep-feature py-14 md:py-20">
      <div className="mx-auto grid w-full max-w-[var(--container-max)] gap-10 px-[var(--gutter-mobile)] md:grid-cols-4 md:px-[var(--gutter-tablet)]">
        <div>
          <Image
            src="/logo.png"
            alt="Sudatta's"
            width={300}
            height={170}
            className="h-24 w-auto md:h-28"
          />
          <p className="mt-5 max-w-xs text-sm leading-relaxed text-[var(--color-on-deep-muted)]">
            Handcrafted Indian occasionwear that feels timeless, elegant, and
            effortlessly wearable.
          </p>
        </div>

        <div>
          <Kicker tone="accent">Shop</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-on-deep-muted)]">
            <li>
              {goTo ? (
                <button
                  type="button"
                  onClick={() => goTo("collections", false)}
                  className="hover:text-[var(--color-on-deep)]"
                >
                  Collections
                </button>
              ) : (
                <Link href="/collections" className="hover:text-[var(--color-on-deep)]">
                  Collections
                </Link>
              )}
            </li>
            <li>
              {goTo ? (
                <button
                  type="button"
                  onClick={() => goTo("shop", false)}
                  className="hover:text-[var(--color-on-deep)]"
                >
                  New Arrivals
                </button>
              ) : (
                <Link href="/#shop" className="hover:text-[var(--color-on-deep)]">
                  New Arrivals
                </Link>
              )}
            </li>
            <li>
              <Link href="/contact-support" className="hover:text-[var(--color-on-deep)]">
                Styling Support
              </Link>
            </li>
            <li>
              <Link href="/about" className="hover:text-[var(--color-on-deep)]">
                About Sudatta&apos;s
              </Link>
            </li>
          </ul>
        </div>

        <div>
          <Kicker tone="accent">Policies</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-on-deep-muted)]">
            <li><Link href="/shipping-policy" className="hover:text-[var(--color-on-deep)]">Shipping Policy</Link></li>
            <li><Link href="/returns-exchanges" className="hover:text-[var(--color-on-deep)]">Returns & Exchanges</Link></li>
            <li><Link href="/terms-conditions" className="hover:text-[var(--color-on-deep)]">Terms & Conditions</Link></li>
            <li><Link href="/privacy-policy" className="hover:text-[var(--color-on-deep)]">Privacy Policy</Link></li>
            <li><Link href="/cancellation-policy" className="hover:text-[var(--color-on-deep)]">Cancellation Policy</Link></li>
            <li><Link href="/payment-guide" className="hover:text-[var(--color-on-deep)]">COD & Prepaid Guide</Link></li>
            <li><Link href="/size-fit-guide" className="hover:text-[var(--color-on-deep)]">Size & Fit Guide</Link></li>
          </ul>
        </div>

        <div>
          <Kicker tone="accent">Contact</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-on-deep-muted)]">
            <li>Order support available from your account orders.</li>
            <li>
              <Link href="/contact-support" className="hover:text-[var(--color-on-deep)]">
                Support, styling, and bulk enquiries
              </Link>
            </li>
            <li>Email support@sudattas.com for quick assistance.</li>
          </ul>
          <div className="mt-6 flex items-center gap-2 text-xs text-[var(--color-on-deep-muted)]">
            <span className="h-px w-5 bg-[var(--color-gold)]" />
            (c) {new Date().getFullYear()} Sudatta&apos;s.
          </div>
        </div>
      </div>
    </footer>
  );
}
