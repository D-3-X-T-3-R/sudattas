import type { Metadata } from "next";
import { headers } from "next/headers";
import {
  STOREFRONT_GATE_REASON_HEADER,
  normalizeStorefrontGateReason,
} from "@/lib/storefront-readiness";
import styles from "./storefront-coming-soon.module.css";

export const metadata: Metadata = {
  title: "We are on our way | Sudatta's",
  description:
    "Sudatta's boutique storefront is being prepared with care and will open soon.",
  robots: {
    index: false,
    follow: false,
    googleBot: {
      index: false,
      follow: false,
    },
  },
};

const bodyCopy = {
  "not-ready":
    "Our boutique storefront is being prepared with care. Soon, you will be able to explore curated sarees, thoughtful details, and pieces chosen for moments that matter.",
  "service-unavailable":
    "Our boutique is temporarily unavailable while we prepare things behind the scenes. Please check back shortly.",
};

function searchParamValue(value: string | string[] | undefined): string | undefined {
  return Array.isArray(value) ? value[0] : value;
}

export default async function StorefrontComingSoonPage({
  searchParams,
}: {
  searchParams?: Promise<{ reason?: string | string[] }>;
}) {
  const params = searchParams ? await searchParams : {};
  const requestHeaders = await headers();
  const reason = normalizeStorefrontGateReason(
    searchParamValue(params.reason) ??
      requestHeaders.get(STOREFRONT_GATE_REASON_HEADER)
  );

  return (
    <main className={styles.page}>
      <div className={styles.fabricField} aria-hidden="true">
        <span className={styles.printLayer} />
        <span className={styles.weftLayer} />
        <span className={styles.goldMeasure} />
        <span className={styles.threadOne} />
        <span className={styles.threadTwo} />
        <span className={styles.threadThree} />
      </div>

      <section className={styles.content} aria-labelledby="coming-soon-title">
        <p className={styles.eyebrow}>SUDATTA&apos;S DESIGNER BOUTIQUE</p>
        <h1 id="coming-soon-title" className={styles.heading}>
          We are on our way.
        </h1>

        <div className={styles.shimmer} aria-hidden="true" />

        <p className={styles.body}>{bodyCopy[reason]}</p>
        <p className={styles.note}>
          For assistance, please check back soon or contact us directly.
        </p>
      </section>
    </main>
  );
}
