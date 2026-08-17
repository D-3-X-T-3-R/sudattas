import type { NextConfig } from "next";

// Product images from your CDN/R2: set NEXT_PUBLIC_IMAGE_HOST to your image host (e.g. pub-xxx.r2.dev or cdn.sudattas.com)
const imageHost = process.env.NEXT_PUBLIC_IMAGE_HOST;

// Baseline security headers. The CSP allow-lists Google sign-in and Razorpay checkout, the two
// third-party origins this app actually embeds — verify against both flows (and any CDN/image
// host beyond the two already known) after deploying, since a too-strict CSP fails silently in
// the browser console rather than at build time.
//
// 'unsafe-eval' is deliberately NOT included: Next.js production builds don't need it, and
// combined with 'unsafe-inline' it would neutralize most of CSP's XSS-mitigation value.
// object-src/base-uri close the classic <object>/<base> injection vectors CSP doesn't block by
// default. frame-ancestors 'self' (CSP, honored by modern browsers) and X-Frame-Options:
// SAMEORIGIN (fallback for older browsers) are kept in agreement rather than contradicting
// each other (frame-ancestors 'self' + X-Frame-Options: DENY previously disagreed).
const contentSecurityPolicy = [
  "default-src 'self'",
  "script-src 'self' 'unsafe-inline' https://accounts.google.com https://checkout.razorpay.com https://*.razorpay.com",
  "style-src 'self' 'unsafe-inline'",
  `img-src 'self' data: https://images.unsplash.com https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev https://*.r2.dev https://*.razorpay.com${imageHost ? ` https://${imageHost}` : ""}`,
  "font-src 'self' data:",
  // r2.cloudflarestorage.com (not r2.dev — that's the public *viewing* CDN host, already in
  // img-src) is the S3-compatible API host the browser PUTs directly to for a presigned image
  // upload (see uploadImageMutation in imtheboss/products/page.tsx). Without it here, every
  // image upload fails at the browser's CSP layer before the request is even sent.
  "connect-src 'self' https://accounts.google.com https://checkout.razorpay.com https://*.razorpay.com https://*.r2.cloudflarestorage.com",
  "frame-src https://accounts.google.com https://checkout.razorpay.com https://*.razorpay.com",
  "frame-ancestors 'self'",
  "object-src 'none'",
  "base-uri 'self'",
].join("; ");

const nextConfig: NextConfig = {
  async headers() {
    return [
      {
        source: "/:path*",
        headers: [
          { key: "X-Frame-Options", value: "SAMEORIGIN" },
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          { key: "Content-Security-Policy", value: contentSecurityPolicy },
        ],
      },
    ];
  },
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "images.unsplash.com",
        pathname: "/**",
      },
      {
        protocol: "https",
        hostname: "pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev",
        pathname: "/**",
      },
      ...(imageHost
        ? [
            {
              protocol: "https" as const,
              hostname: imageHost,
              pathname: "/**",
            },
          ]
        : []),
    ],
  },
};

export default nextConfig;
