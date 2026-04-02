import type { Metadata } from "next";
import { Inter, Playfair_Display } from "next/font/google";
import "./globals.css";
import { Providers } from "@/components/providers";
import { StorefrontProvider } from "@/context/storefront-context";
import { AppErrorBoundary } from "@/components/app-error-boundary";

/** Headings: Playfair Display */
const playfair = Playfair_Display({
  variable: "--font-display",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

/** Body / UI: Inter */
const sans = Inter({
  variable: "--font-sans",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Sudatta's | Designer Sarees",
  description: "Minimal luxury storefront for Sudatta's sarees.",
};

function siteUrl(): string {
  const raw = process.env.NEXT_PUBLIC_SITE_URL?.trim();
  if (!raw) return "https://www.sudattas.com";
  return raw.endsWith("/") ? raw.slice(0, -1) : raw;
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const base = siteUrl();
  const organizationJsonLd = {
    "@context": "https://schema.org",
    "@type": "Organization",
    name: "Sudatta's",
    url: base,
    logo: `${base}/logo.svg`,
    sameAs: [base],
  };

  return (
    <html lang="en">
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(organizationJsonLd) }}
        />
      </head>
      <body
        className={`${playfair.variable} ${sans.variable} font-sans antialiased bg-[var(--background)] text-[var(--foreground)]`}
      >
        <div className="storefront-root min-h-screen w-full min-w-0">
          <Providers>
            <AppErrorBoundary>
              <StorefrontProvider>{children}</StorefrontProvider>
            </AppErrorBoundary>
          </Providers>
        </div>
      </body>
    </html>
  );
}
