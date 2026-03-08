import type { Metadata } from "next";
import { Cormorant_Garamond, Inter, Playfair_Display } from "next/font/google";
import "./globals.css";
import { Providers } from "@/components/providers";
import { StorefrontProvider } from "@/context/storefront-context";

const display = Cormorant_Garamond({
  variable: "--font-display",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

const playfair = Playfair_Display({
  variable: "--font-hero",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

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
        className={`${display.variable} ${playfair.variable} ${sans.variable} font-sans antialiased bg-[var(--background)] text-[var(--foreground)]`}
      >
        <Providers>
        <StorefrontProvider>{children}</StorefrontProvider>
      </Providers>
      </body>
    </html>
  );
}
