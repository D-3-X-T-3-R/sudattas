import type { Metadata } from "next";
import { Plus_Jakarta_Sans, Playfair_Display } from "next/font/google";
import "./globals.css";
import { Providers } from "@/components/providers";
import { StorefrontProvider } from "@/context/storefront-context";
import { AppErrorBoundary } from "@/components/app-error-boundary";
import { SiteHeader } from "@/components/site-header";
import { siteUrl } from "@/lib/site-url";

/** Headings: Playfair Display */
const playfair = Playfair_Display({
  variable: "--font-display",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

/** Body / UI: Plus Jakarta Sans */
const sans = Plus_Jakarta_Sans({
  variable: "--font-sans",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

export const metadata: Metadata = {
  title: "Sudatta's | Designer Sarees",
  description: "Minimal luxury storefront for Sudatta's sarees.",
};

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
    logo: `${base}/logo.png`,
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
              <StorefrontProvider>
                <SiteHeader />
                {children}
              </StorefrontProvider>
            </AppErrorBoundary>
          </Providers>
        </div>
      </body>
    </html>
  );
}
