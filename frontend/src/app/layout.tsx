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

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
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
