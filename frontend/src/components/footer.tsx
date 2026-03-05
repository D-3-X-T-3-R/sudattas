"use client";

import { Kicker } from "@/components/ui/typography";

interface FooterProps {
  goTo: (id: string, instant?: boolean) => void;
}

export function Footer({ goTo }: FooterProps) {
  return (
    <footer className="border-t border-[var(--color-line)] py-14 sm:py-16 md:py-24">
      <div className="mx-auto grid max-w-7xl gap-12 px-4 md:grid-cols-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="h-px w-6 bg-[var(--color-accent-gold)]" />
            <Kicker className="text-[var(--color-ink)]">Sudatta&apos;s</Kicker>
          </div>
          <div className="mt-1 text-[10px] tracking-[0.22em] text-[var(--color-muted)]">
            DESIGNER BOUTIQUE
          </div>
          <p className="mt-4 text-sm leading-relaxed text-[var(--color-muted)]">
            Replace the placeholder images with your saree photos and host them
            (S3/Cloudflare R2/etc.) for production.
          </p>
        </div>
        <div>
          <Kicker className="text-[var(--color-ink)]">Shop</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li>
              <button
                type="button"
                onClick={() => goTo("collections", false)}
                className="transition-colors hover:text-[var(--color-accent-brown)]"
              >
                Collections
              </button>
            </li>
            <li>
              <button
                type="button"
                onClick={() => goTo("shop", false)}
                className="transition-colors hover:text-[var(--color-accent-brown)]"
              >
                New arrivals
              </button>
            </li>
            <li>Gift cards</li>
          </ul>
        </div>
        <div>
          <Kicker className="text-[var(--color-ink)]">Services</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li>Shipping & delivery</li>
            <li>Returns & exchanges</li>
            <li>Care guide</li>
          </ul>
        </div>
        <div>
          <Kicker className="text-[var(--color-ink)]">Contact</Kicker>
          <ul className="mt-4 space-y-2 text-sm text-[var(--color-muted)]">
            <li>support@sudattas.com</li>
            <li>+91 90000 00000</li>
            <li>Instagram: @sudattas</li>
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
