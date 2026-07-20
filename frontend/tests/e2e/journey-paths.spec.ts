import path from "node:path";
import { spawnSync } from "node:child_process";
import { expect, test } from "@playwright/test";
import {
  installCommerceMocks,
  prepareAuthenticatedBag,
  type CommerceMockOptions,
} from "./helpers/journey-commerce-mocks";

type BucketMode = "browser" | "catalog_only";

type BucketPlan = {
  mode: BucketMode;
  reason?: string;
};

type CatalogReport = {
  totals: {
    totalJourneysParsed: number;
    blueprintRows: number;
    extendedRows: number;
    mappedCount: number;
    catalogOnlyCount: number;
    browserRepresentedCount: number;
    providerLiveExcluded: number;
    unknownUnmappedCount: number;
    providerLeakFindings: number;
    unknownPathFindings: number;
  };
  bucketCounts: Record<string, number>;
  bucketModes: Record<string, BucketPlan>;
  requiredBuckets: string[];
  sampleIdsByBucket: Record<string, string[]>;
  idToBucket: Record<string, string>;
};

const REPO_ROOT = path.resolve(__dirname, "../../../");

const REQUIRED_BUCKETS = [
  "entry_discovery",
  "catalog_browsing",
  "product_detail",
  "guest_session",
  "cart",
  "checkout",
  "payment_states",
  "auth",
  "account_orders",
  "account_refunds",
  "wishlist",
  "admin_auth",
  "admin_catalog",
  "admin_orders",
  "admin_customers",
  "security_negative",
  "reliability_error",
  "accessibility_smoke",
  "seo_content",
  "responsive_mobile",
] as const;

const BUCKET_PLAN: Record<(typeof REQUIRED_BUCKETS)[number], BucketPlan> = {
  entry_discovery: { mode: "browser" },
  catalog_browsing: { mode: "browser" },
  product_detail: {
    mode: "catalog_only",
    reason:
      "PDP route is server-fetched and needs deterministic seeded product fixtures in this pipeline; validated by catalog mapping and component tests.",
  },
  guest_session: { mode: "browser" },
  cart: { mode: "browser" },
  checkout: { mode: "browser" },
  payment_states: { mode: "browser" },
  auth: { mode: "browser" },
  account_orders: { mode: "browser" },
  account_refunds: {
    mode: "catalog_only",
    reason:
      "Refund permutations depend on historical order/refund datasets and are validated via API/unit coverage plus catalog validation.",
  },
  wishlist: { mode: "browser" },
  admin_auth: { mode: "browser" },
  admin_catalog: {
    mode: "catalog_only",
    reason:
      "Admin catalog CRUD requires seeded privileged datasets and is covered by admin unit/contract tests.",
  },
  admin_orders: {
    mode: "catalog_only",
    reason:
      "Admin order/refund/dispute reconciliation requires backend-heavy seeded states and is covered by backend integration plus catalog validation.",
  },
  admin_customers: {
    mode: "catalog_only",
    reason:
      "Admin customer/audit reporting surfaces require seeded privileged datasets and are contract-tested.",
  },
  security_negative: { mode: "browser" },
  reliability_error: { mode: "browser" },
  accessibility_smoke: { mode: "browser" },
  seo_content: {
    mode: "catalog_only",
    reason:
      "SEO/static surfaces are validated by sitemap/robots/unit tests and catalog scripts.",
  },
  responsive_mobile: { mode: "browser" },
};

let reportCache: CatalogReport | null = null;

function runNodeScript(scriptRelativePath: string, args: string[] = []) {
  const scriptPath = path.resolve(REPO_ROOT, scriptRelativePath);
  return spawnSync("node", [scriptPath, ...args], {
    cwd: REPO_ROOT,
    encoding: "utf8",
  });
}

function getCatalogReport(): CatalogReport {
  if (reportCache) return reportCache;

  const result = runNodeScript("scripts/check-journey-catalog-mapping.mjs", ["--json"]);
  expect(result.status, `catalog mapping script failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`).toBe(0);

  const parsed = JSON.parse(result.stdout.trim()) as CatalogReport | { ok: false; error: string };
  if ("ok" in parsed && parsed.ok === false) {
    throw new Error(`Catalog mapping script returned error payload: ${parsed.error}`);
  }

  reportCache = parsed as CatalogReport;
  return reportCache;
}

// Desktop shows "Proceed To Checkout" (order-summary-panel.tsx); mobile viewports show the
// sticky "Checkout (N)" bar instead (bag-mobile-checkout-bar.tsx). Both can be present in the
// DOM at once (one hidden per breakpoint), so this must match either name AND be visible --
// this file runs under both the desktop and mobile Playwright projects depending on invocation.
function checkoutButton(page: Parameters<typeof installCommerceMocks>[0]) {
  return page
    .getByRole("button", { name: /^(Proceed To Checkout|Checkout \(\d+\))$/i })
    .and(page.locator(":visible"));
}

async function runCheckoutFlowToCompletion(
  page: Parameters<typeof installCommerceMocks>[0],
  options?: CommerceMockOptions
) {
  const mocks = await installCommerceMocks(page, options);
  await prepareAuthenticatedBag(page, mocks);
  await checkoutButton(page).first().click();
  return mocks;
}

test("catalog validation: bucket plan is complete and documented", async () => {
  expect(Object.keys(BUCKET_PLAN).sort()).toEqual([...REQUIRED_BUCKETS].sort());

  for (const bucket of REQUIRED_BUCKETS) {
    const plan = BUCKET_PLAN[bucket];
    expect(plan, `Missing plan for bucket '${bucket}'`).toBeDefined();
    if (plan.mode === "catalog_only") {
      expect(plan.reason?.trim().length ?? 0, `Catalog-only bucket '${bucket}' must include a reason`).toBeGreaterThan(0);
    }
  }
});

test("catalog validation: parses all rows, maps all rows, and reports coverage", async () => {
  const report = getCatalogReport();

  expect(report.totals.blueprintRows).toBeGreaterThanOrEqual(220);
  expect(report.totals.extendedRows).toBeGreaterThanOrEqual(5000);
  expect(report.totals.totalJourneysParsed).toBe(report.totals.blueprintRows + report.totals.extendedRows);
  expect(report.totals.mappedCount).toBe(report.totals.totalJourneysParsed);
  expect(report.totals.unknownUnmappedCount).toBe(0);
  expect(report.totals.providerLeakFindings).toBe(0);
  expect(report.totals.unknownPathFindings).toBe(0);
  expect(report.totals.providerLiveExcluded).toBeGreaterThan(0);

  expect([...report.requiredBuckets].sort()).toEqual([...REQUIRED_BUCKETS].sort());

  for (const bucket of REQUIRED_BUCKETS) {
    expect(report.bucketCounts[bucket], `Missing bucket count for ${bucket}`).toBeGreaterThan(0);
  }

  console.info("[journey-report]", {
    totalJourneysParsed: report.totals.totalJourneysParsed,
    mappedCount: report.totals.mappedCount,
    catalogOnlyCount: report.totals.catalogOnlyCount,
    browserRepresentedCount: report.totals.browserRepresentedCount,
    providerLiveExcluded: report.totals.providerLiveExcluded,
    unknownUnmappedCount: report.totals.unknownUnmappedCount,
    bucketCounts: report.bucketCounts,
  });
});

test("catalog validation: checkout journey IDs UJ-070..UJ-075 map to checkout bucket", async () => {
  const report = getCatalogReport();
  const checkoutIds = ["UJ-070", "UJ-071", "UJ-072", "UJ-073", "UJ-074", "UJ-075"];
  for (const id of checkoutIds) {
    expect(report.idToBucket[id], `${id} should map to checkout bucket`).toBe("checkout");
  }
});

test("provider-safe leakage guard script passes", async () => {
  const result = runNodeScript("scripts/check-provider-safe-journey-catalogs.mjs");
  expect(result.status, `provider-safe guard failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`).toBe(0);
});

test("bucket: entry_discovery representative harness", async ({ page }) => {
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("button", { name: "Explore" }).first()).toBeVisible();
  await expect(page.getByRole("button", { name: "Search" })).toBeVisible();
  await expect(page.locator("body")).toContainText(/shop|explore|collection/i);
});

test("bucket: catalog_browsing representative harness", async ({ page }) => {
  await installCommerceMocks(page);
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect(page.locator('button[aria-label^="View "]').first()).toBeVisible();

  await page.getByRole("button", { name: "Search" }).click();
  const searchInput = page
    .getByPlaceholder("Search collections, fabrics, styles")
    .and(page.locator(":visible"));
  await expect(searchInput.first()).toBeVisible();
  await searchInput.first().fill("silk");

  await page.getByLabel("Sort").selectOption("Price: Low to High");
  await expect(page.locator("body")).toContainText(/Amber Saree|Emerald Saree/i);
});

test("bucket: guest_session representative harness", async ({ page }) => {
  await installCommerceMocks(page);
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect.poll(async () =>
    page.evaluate(() => window.localStorage.getItem("sudattas_guest_session"))
  ).toBe("guest-session-1");
});

test("bucket: cart representative harness", async ({ page }) => {
  const mocks = await installCommerceMocks(page);
  await prepareAuthenticatedBag(page, mocks);

  await page.goto("/bag", { waitUntil: "domcontentloaded" });
  await expect(page.locator("body")).toContainText("Amber Saree");
  await expect(page.locator("body")).toContainText("Emerald Saree");

  await page.getByRole("button", { name: /Increase quantity for Amber Saree/i }).click();
  await page.getByRole("button", { name: /Decrease quantity for Amber Saree/i }).click();
  await expect(page.locator("body")).toContainText(/Order Summary/i);
});

test("bucket: checkout representative harness (stateful, not empty-bag)", async ({ page }) => {
  const mocks = await installCommerceMocks(page);
  await prepareAuthenticatedBag(page, mocks);

  await page.goto("/checkout/address", { waitUntil: "domcontentloaded" });
  await expect(page).toHaveURL(/\/bag$/);

  await expect(page.locator("body")).not.toContainText("Your bag is empty");
  await expect(page.locator("body")).toContainText(/Delivery & Payment/i);
  await expect(page.locator("body")).toContainText(/Order Summary/i);
  await expect(checkoutButton(page).first()).toBeVisible();
});

test("bucket: payment_states representative harness (paid)", async ({ page }) => {
  await runCheckoutFlowToCompletion(page, { verifyPaymentState: "paid", verifyOrderUiState: "processing" });
  await expect(page).toHaveURL(/\/checkout\/success\?orderId=9001&payment=paid/);
  await expect(page.locator("body")).toContainText(/Your payment is verified/i);
});

test("bucket: payment_states representative harness (failed)", async ({ page }) => {
  await runCheckoutFlowToCompletion(page, { verifyPaymentState: "failed", verifyOrderUiState: "failed" });
  await expect(page).toHaveURL(/\/checkout\/failed/);
  await expect(page.locator("body")).toContainText("Payment was not completed. You can retry safely.");
});

test("bucket: payment_states representative harness (pending)", async ({ page }) => {
  await runCheckoutFlowToCompletion(page, { verifyPaymentState: "pending", verifyOrderUiState: "pending" });
  await expect(page).toHaveURL(/payment=pending/);
  await expect(page.locator("body")).toContainText("We're confirming your payment. Please don't place another order yet.");
  await expect(page.locator("body")).not.toContainText(/payment is verified/i);
});

test("bucket: payment_states representative harness (needs_review)", async ({ page }) => {
  await runCheckoutFlowToCompletion(page, { verifyPaymentState: "paid", verifyOrderUiState: "needs_review" });
  await expect(page).toHaveURL(/payment=needs_review/);
  await expect(page.locator("body")).toContainText(
    "We received your payment update, but it needs manual verification. We'll contact you if action is needed."
  );
  await expect(page.locator("body")).not.toContainText(/payment is verified/i);
});

test("bucket: auth representative harness", async ({ page }) => {
  await installCommerceMocks(page);
  await page.goto("/profile", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("button", { name: /^Sign in$/i })).toBeVisible();
  await expect(page.locator("body")).toContainText("Your Profile");
});

test("bucket: account_orders representative harness", async ({ page }) => {
  await runCheckoutFlowToCompletion(page, { verifyPaymentState: "paid", verifyOrderUiState: "processing" });

  await page.goto("/profile", { waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: "Orders" }).click();
  await expect(page.locator("body")).toContainText("Order #9001");
  await page.getByRole("button", { name: /Refresh tracking/i }).click();
  await expect(page.locator("body")).toContainText("AWB AWB-123");
});

test("bucket: wishlist representative harness", async ({ page }) => {
  const mocks = await installCommerceMocks(page);
  mocks.setAuthenticated(true);

  await page.goto("/wishlist", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("heading", { name: "Saved Favourites" })).toBeVisible();
  await expect(page.locator("body")).toContainText("Amber Saree");
});

test("bucket: admin_auth representative harness", async ({ page }) => {
  await page.goto("/imtheboss/orders", { waitUntil: "domcontentloaded" });
  await expect(page).toHaveURL(/\/imtheboss\/login/);
  await expect(
    page.getByRole("button", { name: /^(Sign in(?: with Google)?|Try another account)$/i }).first()
  ).toBeVisible();
});

test("bucket: security_negative representative harness", async ({ page, request }) => {
  const unauthorizedProfile = await request.get("/api/account/profile", { failOnStatusCode: false });
  expect([401, 403].includes(unauthorizedProfile.status())).toBeTruthy();

  await page.goto("/imtheboss/orders", { waitUntil: "domcontentloaded" });
  await expect(page).toHaveURL(/\/imtheboss\/login/);
});

test("bucket: reliability_error representative harness", async ({ page }) => {
  await page.route("**/api/products**", async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ products: [], error: "service unavailable" }),
    });
  });

  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect(page.locator("body")).toContainText(/Catalog temporarily unavailable/i);
});

test("bucket: accessibility_smoke representative harness", async ({ page }) => {
  const mocks = await installCommerceMocks(page);
  await prepareAuthenticatedBag(page, mocks);

  const addAddressButton = page.getByRole("button", { name: "Add New" }).first();
  await addAddressButton.focus();
  await addAddressButton.click();

  await expect(page.getByLabel("Full name")).toBeVisible();
  await expect(page.getByLabel("Phone")).toBeVisible();
  await expect(page.getByLabel("Address line")).toBeVisible();
  await expect(page.getByLabel("City")).toBeVisible();
  await expect(page.getByLabel("State")).toBeVisible();
  await expect(page.getByLabel("Country")).toBeVisible();
  await expect(page.getByLabel("Postal code")).toBeVisible();

  await expect(page.locator("#bag-address-recipient-name")).toBeFocused();
  await page.keyboard.press("Tab");
  await page.keyboard.press("Tab");

  const focusInsideDialog = await page.evaluate(() => {
    const dialog = document.querySelector("[role='dialog']");
    return dialog ? dialog.contains(document.activeElement) : false;
  });
  expect(focusInsideDialog).toBeTruthy();

  await page.keyboard.press("Escape");
  await expect(addAddressButton).toBeFocused();
});

test("bucket: responsive_mobile representative harness", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const mocks = await installCommerceMocks(page);
  await prepareAuthenticatedBag(page, mocks);

  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("button", { name: /Open menu/i })).toBeVisible();

  await page.goto("/bag", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("button", { name: /Checkout \(/i })).toBeVisible();
});
