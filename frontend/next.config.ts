import type { NextConfig } from "next";

// Product images from your CDN/R2: set NEXT_PUBLIC_IMAGE_HOST to your image host (e.g. pub-xxx.r2.dev or cdn.sudattas.com)
const imageHost = process.env.NEXT_PUBLIC_IMAGE_HOST;

const nextConfig: NextConfig = {
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
