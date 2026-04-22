"use client";

import Image from "next/image";
import Link from "next/link";
import { Kicker } from "@/components/ui/typography";

interface FooterProps {
  goTo: (id: string, instant?: boolean) => void;
}

export function Footer({ goTo }: FooterProps) {
  return (
    <footer className="border-t border-[var(--color-line)] py-14 sm:py-16 md:py-24">
      <div className="mx-auto grid max-w-[2000px] gap-12 px-4 md:grid-cols-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="h-px w-6 bg-[var(--color-accent-gold)]" />
            <Image
              src="/Sudattas_logo_final_transparent.png"
              alt="Sudatta's"
              width={140}
              height={42}
              className="h-7 w-auto"
            />
          </div>
          <div className="mt-1 text-[10px] tracking-[0.22em] text-[var(--color-muted)]">
            DESIGNER BOUTIQUE
          </div>
          <p className="mt-4 text-sm leading-relaxed text-[var(--color-muted)]">
            Handcrafted sarees with timeless drape, modern detail, and
            occasion-ready styling.
          </p>
        </div>
        <div>
          <Kicker className="text-[var(--color-ink)]">Shop</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li>
              <button
                type="button"
                onClick={() => goTo("collections", false)}
                className="transition-colors hover:text-[var(--color-accent-gold)]"
              >
                Collections
              </button>
            </li>
            <li>
              <button
                type="button"
                onClick={() => goTo("shop", false)}
                className="transition-colors hover:text-[var(--color-accent-gold)]"
              >
                New arrivals
              </button>
            </li>
            <li>
              <Link
                href="/contact-support"
                className="transition-colors hover:text-[var(--color-accent-gold)]"
              >
                Styling support
              </Link>
            </li>
          </ul>
        </div>
        <div>
          <Kicker className="text-[var(--color-ink)]">Services</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li>
              <Link
                href="/shipping-policy"
                className="transition-colors hover:text-[var(--color-accent-gold)]"
              >
                Shipping Policy
              </Link>
            </li>
            <li>
              <Link
                href="/returns-exchanges"
                className="transition-colors hover:text-[var(--color-accent-gold)]"
              >
                Returns & Exchanges
              </Link>
            </li>
            <li>
              <Link
                href="/terms-conditions"
                className="transition-colors hover:text-[var(--color-accent-gold)]"
              >
                Terms & Conditions
              </Link>
            </li>
            <li>
              <Link
                href="/privacy-policy"
                className="transition-colors hover:text-[var(--color-accent-gold)]"
              >
                Privacy Policy
              </Link>
            </li>
          </ul>
        </div>
        <div>
          <Kicker className="text-[var(--color-ink)]">Contact</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li>Customer support available through your account orders.</li>
            <li>
              <Link
                href="/contact-support"
                className="transition-colors hover:text-[var(--color-accent-gold)]"
              >
                Styling, support, and bulk enquiries
              </Link>
            </li>
            <li>Email support@sudattas.com for assisted resolution.</li>
          </ul>
          <div className="mt-6 flex items-center gap-2 text-xs text-[var(--color-muted)]">
            <span className="h-px w-4 bg-[var(--color-accent-gold)]" />
            © {new Date().getFullYear()} Sudatta&apos;s.
          </div>
        </div>
      </div>
    </footer>
  );
}
