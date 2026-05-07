import fs from "node:fs";
import path from "node:path";
import { expect, test, type APIRequestContext, type Page } from "@playwright/test";

type JourneyRow = {
  id: string;
  section?: number;
  journey: string;
  sitePath: string;
  endpoint: string;
  source: string;
};

type JourneySeed = {
  seed: string;
  email: string;
  idempotencyKey: string;
};

const DEFAULT_BLUEPRINTS = [
  path.resolve(__dirname, "../../../docs/user-journeys-blueprint.md"),
  path.resolve(__dirname, "../../../docs/user-journeys-catalog-5000.md"),
];
const PROVIDER_LIVE_CATALOG = path.resolve(
  __dirname,
  "../../../docs/user-journeys-provider-live.md"
);
const DEFERRED_IDS = new Set(["UJ-204", "UJ-205", "UJ-206", "UJ-207"]);
const ROUTE_STATUS_OK = new Set([200, 301, 302, 303, 307, 308, 401, 403, 404]);
const API_STATUS_OK = new Set([200, 201, 204, 400, 401, 403, 404, 405, 409, 422, 429]);
const REQUIRE_BEHAVIOR_ASSERTIONS = process.env.PW_REQUIRE_BEHAVIOR_ASSERTIONS !== "0";
// Default CI is provider-safe. Real provider journeys are opt-in only.
const LIVE_PROVIDER_CONFIRMATION =
  "I_UNDERSTAND_THIS_HITS_REAL_PROVIDERS";
const RUN_LIVE_PROVIDER_JOURNEYS = process.env.RUN_LIVE_PROVIDER_JOURNEYS === "1";
const PROVIDER_LIVE_TEST_CONFIRM = process.env.PROVIDER_LIVE_TEST_CONFIRM?.trim();
const INCLUDE_PROVIDER_LIVE_JOURNEYS =
  RUN_LIVE_PROVIDER_JOURNEYS && PROVIDER_LIVE_TEST_CONFIRM === LIVE_PROVIDER_CONFIRMATION;
if (RUN_LIVE_PROVIDER_JOURNEYS && !INCLUDE_PROVIDER_LIVE_JOURNEYS) {
  throw new Error(
    "RUN_LIVE_PROVIDER_JOURNEYS=1 requires PROVIDER_LIVE_TEST_CONFIRM=I_UNDERSTAND_THIS_HITS_REAL_PROVIDERS"
  );
}

const PROVIDER_LIVE_MARKERS = [
  "@provider-live",
  "@razorpay-live",
  "@shiprocket-live",
  "real razorpay",
  "real shiprocket",
  "razorpay live",
  "shiprocket live",
  "live webhook",
  "provider callback",
  "razorpay capture webhook",
  "razorpay refund webhook",
  "shiprocket booking",
  "shiprocket cancellation",
  "awb callback",
] as const;

const CATALOG_FORBIDDEN_PATTERNS = [
  /@provider-live/i,
  /@razorpay-live/i,
  /@shiprocket-live/i,
  /\breal razorpay\b/i,
  /\breal shiprocket\b/i,
  /\blive webhook\b/i,
  /\bprovider callback\b/i,
  /\bshiprocket cancellation\b/i,
  /\bshiprocket booking\b/i,
  /\bawb callback\b/i,
  /\brazorpay capture webhook\b/i,
  /\brazorpay refund webhook\b/i,
];

const CATALOG_ALLOWED_PROVIDER_CONTEXT =
  /\b(mock-safe|excluded from default ci|provider-live\.md)\b/i;

type ForbiddenCatalogMatch = {
  file: string;
  line: number;
  text: string;
  pattern: string;
};

function parseSectionFilter(): Set<number> | null {
  const raw = process.env.JOURNEY_SECTIONS?.trim();
  if (!raw) return null;
  const nums = raw
    .split(",")
    .map((s) => Number(s.trim()))
    .filter((n) => Number.isFinite(n) && n >= 1 && n <= 16);
  if (nums.length === 0) return null;
  return new Set(nums);
}

function resolveJourneyFiles(): string[] {
  const excludeProviderLiveCatalog = (files: string[]) =>
    INCLUDE_PROVIDER_LIVE_JOURNEYS
      ? files
      : files.filter((filePath) => path.normalize(filePath) !== path.normalize(PROVIDER_LIVE_CATALOG));

  const fromEnv = process.env.JOURNEY_BLUEPRINT_FILES?.trim();
  if (!fromEnv) {
    return excludeProviderLiveCatalog(DEFAULT_BLUEPRINTS.filter((p) => fs.existsSync(p)));
  }
  return excludeProviderLiveCatalog(fromEnv
    .split(",")
    .map((f) => path.resolve(__dirname, "../../../", f.trim()))
    .filter((p) => fs.existsSync(p)));
}

function splitMarkdownRow(line: string): string[] | null {
  if (!line.startsWith("|")) return null;

  const cells: string[] = [];
  let current = "";

  for (let i = 1; i < line.length; i += 1) {
    const ch = line[i];
    const next = line[i + 1];

    if (ch === "\\" && next === "|") {
      current += "|";
      i += 1;
      continue;
    }

    if (ch === "|") {
      cells.push(current.trim().replace(/&#124;/g, "|"));
      current = "";
      continue;
    }

    current += ch;
  }

  if (cells.length === 0) return null;

  const maybeSeparator = cells.every((c) => /^:?-{3,}:?$/.test(c));
  if (maybeSeparator) return null;

  return cells;
}

function parseRowsFromFile(filePath: string): JourneyRow[] {
  const text = fs.readFileSync(filePath, "utf8");
  const lines = text.split(/\r?\n/);
  const rows: JourneyRow[] = [];

  for (const line of lines) {
    const cells = splitMarkdownRow(line);
    if (!cells || cells.length < 4) continue;

    const id = cells[0];
    if (!/^UJ(?:X)?-\d{3,4}$/.test(id)) continue;

    const parsedSection = Number(cells[1]);
    const section = Number.isFinite(parsedSection) ? parsedSection : undefined;
    const journey = cells[2] ?? "";
    const sitePath = cells[3] ?? "";
    const endpoint = cells[4] ?? "";

    rows.push({
      id,
      section,
      journey,
      sitePath,
      endpoint,
      source: path.basename(filePath),
    });
  }

  return rows;
}

function parseRows(): JourneyRow[] {
  const files = resolveJourneyFiles();
  const all = files.flatMap(parseRowsFromFile);
  const unique = new Map<string, JourneyRow>();

  for (const row of all) {
    if (!unique.has(row.id)) unique.set(row.id, row);
  }

  return [...unique.values()].sort((a, b) => a.id.localeCompare(b.id));
}

function isProviderLiveRow(row: JourneyRow): boolean {
  const text = `${row.journey} ${row.sitePath} ${row.endpoint}`.toLowerCase();
  return PROVIDER_LIVE_MARKERS.some((marker) => text.includes(marker));
}

function collectForbiddenCatalogMatches(filePath: string): ForbiddenCatalogMatch[] {
  if (!fs.existsSync(filePath)) return [];
  const text = fs.readFileSync(filePath, "utf8");
  const lines = text.split(/\r?\n/);
  const findings: ForbiddenCatalogMatch[] = [];

  lines.forEach((line, index) => {
    for (const pattern of CATALOG_FORBIDDEN_PATTERNS) {
      if (!pattern.test(line)) continue;
      if (CATALOG_ALLOWED_PROVIDER_CONTEXT.test(line)) continue;
      findings.push({
        file: path.basename(filePath),
        line: index + 1,
        text: line.trim(),
        pattern: pattern.toString(),
      });
    }
  });

  return findings;
}

function journeySeed(id: string): JourneySeed {
  const compact = id.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
  return {
    seed: compact,
    email: `user-${compact}@test.com`,
    idempotencyKey: `journey-${compact}`,
  };
}

function normalizePath(raw: string): string | null {
  let p = raw.trim();
  if (!p) return null;
  p = p.replace(/`/g, "");
  if (p.startsWith("http://") || p.startsWith("https://")) return null;
  if (!p.startsWith("/")) return null;

  p = p.replace(/\[slug\]/gi, "sample-slug");
  p = p.replace(/\[id\]/gi, "1");
  p = p.replace(/\[categoryId\]/gi, "1");
  p = p.replace(/\[orderId\]/gi, "1");
  p = p.replace(/\*/g, "");
  p = p.replace(/\/+/g, "/");

  if (p === "/products") return "/";
  if (p.startsWith("/products/")) return `/product/${p.split("/").pop() || "1"}`;
  if (p === "/cart") return "/bag";
  if (p.startsWith("/account/")) return "/profile";
  if (p === "/account") return "/profile";
  if (p === "/checkout") return "/checkout/address";
  if (p === "/checkout/payment") return "/checkout/address";
  if (p === "/login") return "/";

  if (p === "/admin" || p === "/admin/") return "/imtheboss";
  if (p.startsWith("/admin/")) return `/imtheboss${p.slice("/admin".length)}`;
  if (p === "/account/measurements") return "/profile";
  if (p === "/api/checkout") return "/api/checkout/place-order";
  if (p === "/api/account") return "/api/account/profile";

  return p;
}

function extractPaths(cell: string): string[] {
  const paths = new Set<string>();

  for (const m of cell.matchAll(/`([^`]+)`/g)) {
    const p = normalizePath(m[1]);
    if (p) paths.add(p);
  }

  for (const m of cell.matchAll(/(\/[-A-Za-z0-9_./\[\]*]+)/g)) {
    const p = normalizePath(m[1]);
    if (p) paths.add(p);
  }

  return [...paths];
}

function seedHeaders(seed: JourneySeed): Record<string, string> {
  return {
    "x-journey-seed": seed.seed,
    "x-journey-email": seed.email,
  };
}

async function getWithSeed(request: APIRequestContext, target: string, seed: JourneySeed) {
  return request.get(target, {
    failOnStatusCode: false,
    headers: seedHeaders(seed),
  });
}

async function postWithSeed(
  request: APIRequestContext,
  target: string,
  seed: JourneySeed,
  data: Record<string, unknown>,
  headers: Record<string, string> = {}
) {
  return request.post(target, {
    failOnStatusCode: false,
    data,
    headers: {
      ...seedHeaders(seed),
      ...headers,
    },
  });
}

async function assertRoutePath(pathValue: string, page: Page, label: string) {
  const res = await page.goto(pathValue, { waitUntil: "domcontentloaded" });
  if (!res) return;
  expect(ROUTE_STATUS_OK.has(res.status()), `${label} ${pathValue} -> ${res.status()}`).toBeTruthy();
}

async function assertApiEnvelope(
  res: Awaited<ReturnType<APIRequestContext["get"]>>,
  label: string
) {
  const status = res.status();
  expect(API_STATUS_OK.has(status), `${label} unexpected status ${status}`).toBeTruthy();
  const ctype = res.headers()["content-type"] || "";
  if (!ctype.includes("application/json")) return;
  const body = await res.json().catch(() => null);
  if (!body || typeof body !== "object") return;
  if ("ok" in body) expect(typeof (body as { ok: unknown }).ok).toBe("boolean");
}

async function goHome(page: Page) {
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("button", { name: "Search" })).toBeVisible();
}

async function tryOpenProductFromHome(page: Page): Promise<boolean> {
  await goHome(page);
  const quickView = page.locator('button[aria-label^="Quick view "]').first();
  const count = await quickView.count();
  if (count === 0) return false;
  await quickView.click();
  await expect(page).toHaveURL(/\/product\//);
  await expect(page.getByRole("button", { name: "ADD TO BAG" })).toBeVisible();
  return true;
}

async function tryOpenProductDirectFromJourney(row: JourneyRow, page: Page): Promise<boolean> {
  const candidates = extractPaths(`${row.sitePath} ${row.endpoint}`).filter((p) => p.startsWith("/product/"));
  for (const candidate of candidates) {
    const res = await page.goto(candidate, { waitUntil: "domcontentloaded" });
    if (!res) continue;
    if (res.status() !== 200) continue;
    if ((await page.getByRole("button", { name: /add to bag/i }).count()) > 0) {
      return true;
    }
  }
  return false;
}

async function openPdpForJourney(row: JourneyRow, page: Page): Promise<"listing" | "direct" | null> {
  const journey = row.journey.toLowerCase();
  if (journey.includes("from listing")) {
    const fromListing = await tryOpenProductFromHome(page);
    return fromListing ? "listing" : null;
  }

  const direct = await tryOpenProductDirectFromJourney(row, page);
  if (direct) return "direct";

  const fromListing = await tryOpenProductFromHome(page);
  if (fromListing) return "listing";

  return null;
}

async function runSection11Scenario(journey: string, request: APIRequestContext, seed: JourneySeed) {
  if (journey.includes("rate limit")) {
    const res = await getWithSeed(request, "/api/products", seed);
    expect([200, 429, 503].includes(res.status())).toBeTruthy();
    return;
  }

  if (journey.includes("idempotency")) {
    const first = await postWithSeed(
      request,
      "/api/account/cart/merge",
      seed,
      {},
      { "Idempotency-Key": `${seed.idempotencyKey}-same` }
    );
    const second = await postWithSeed(
      request,
      "/api/account/cart/merge",
      seed,
      {},
      { "Idempotency-Key": `${seed.idempotencyKey}-same` }
    );
    expect(API_STATUS_OK.has(first.status())).toBeTruthy();
    expect(API_STATUS_OK.has(second.status())).toBeTruthy();
  }
}

async function runSection15Scenario(journey: string, request: APIRequestContext, seed: JourneySeed) {
  if (journey.includes("robots") || journey.includes("sitemap")) {
    const robots = await getWithSeed(request, "/robots.txt", seed);
    const sitemap = await getWithSeed(request, "/sitemap.xml", seed);
    expect([200, 404].includes(robots.status())).toBeTruthy();
    expect([200, 404].includes(sitemap.status())).toBeTruthy();
  }
}

async function runSection16Scenario(page: Page) {
  await page.setViewportSize({ width: 320, height: 800 });
  await goHome(page);
  await expect(page.locator("body")).toBeVisible();
  await page.setViewportSize({ width: 1280, height: 720 });
}

type BehaviorCheck =
  | "home_surface"
  | "top_nav_surface"
  | "footer_nav_surface"
  | "deep_link_surface"
  | "search_surface"
  | "product_detail_surface"
  | "quantity_surface"
  | "image_gallery_surface"
  | "related_products_surface"
  | "bag_surface"
  | "checkout_surface"
  | "profile_surface"
  | "wishlist_surface"
  | "orders_surface"
  | "refund_surface"
  | "auth_signin_surface"
  | "guest_capabilities"
  | "admin_guard_surface"
  | "payment_verify_surface"
  | "telemetry_surface"
  | "not_found_surface"
  | "device_surface";

function collectBehaviorChecks(row: JourneyRow): BehaviorCheck[] {
  const checks = new Set<BehaviorCheck>();
  const journeyText = row.journey.toLowerCase();
  const pathText = `${row.sitePath} ${row.endpoint}`.toLowerCase();
  const text = `${journeyText} ${pathText}`;

  if (text.includes("home")) checks.add("home_surface");
  if (journeyText.includes("top nav")) checks.add("top_nav_surface");
  if (journeyText.includes("footer")) checks.add("footer_nav_surface");
  if (journeyText.includes("deep link") || journeyText.includes("direct url") || pathText.includes("[slug]")) {
    checks.add("deep_link_surface");
  }
  if (text.includes("search") || text.includes("filter") || text.includes("sort")) checks.add("search_surface");
  if (journeyText.includes("pdp") || journeyText.includes("product detail") || journeyText.includes("variant")) {
    checks.add("product_detail_surface");
  }
  if (journeyText.includes("quantity")) checks.add("quantity_surface");
  if (journeyText.includes("image gallery") || journeyText.includes("missing image")) checks.add("image_gallery_surface");
  if (journeyText.includes("related products")) checks.add("related_products_surface");
  if (text.includes("bag") || text.includes("cart")) checks.add("bag_surface");
  if (text.includes("checkout")) checks.add("checkout_surface");
  if (text.includes("profile") || text.includes("account")) checks.add("profile_surface");
  if (text.includes("wishlist")) checks.add("wishlist_surface");
  if (text.includes("order")) checks.add("orders_surface");
  if (text.includes("refund")) checks.add("refund_surface");
  if (
    journeyText.includes("login")
    || journeyText.includes("sign in")
    || journeyText.includes("oauth")
    || journeyText.includes("otp")
    || journeyText.includes("auth")
  ) {
    checks.add("auth_signin_surface");
  }
  if (text.includes("guest") || text.includes("session")) checks.add("guest_capabilities");
  if (text.includes("admin")) checks.add("admin_guard_surface");
  if (text.includes("payment")) checks.add("payment_verify_surface");
  if (text.includes("telemetry")) checks.add("telemetry_surface");
  if (text.includes("non-existent") || text.includes("unknown path") || text.includes("404")) checks.add("not_found_surface");
  if (
    text.includes("chrome")
    || text.includes("safari")
    || text.includes("firefox")
    || text.includes("edge")
    || text.includes("ios")
    || text.includes("android")
    || text.includes("viewport")
    || text.includes("tablet")
    || text.includes("mobile")
    || text.includes("locale")
  ) {
    checks.add("device_surface");
  }

  return [...checks];
}

async function runBehaviorChecks(
  row: JourneyRow,
  checks: BehaviorCheck[],
  page: Page,
  request: APIRequestContext,
  seed: JourneySeed
) {
  for (const check of checks) {
    if (check === "home_surface") {
      await goHome(page);
      await expect(page.locator("body")).toContainText(/explore|shop|collection|sign in/i);
      continue;
    }

    if (check === "top_nav_surface") {
      await goHome(page);
      await page.getByRole("button", { name: "Explore" }).first().click();
      await expect(page.locator("#explore")).toHaveCount(1);
      continue;
    }

    if (check === "footer_nav_surface") {
      await goHome(page);
      await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
      await page.getByRole("button", { name: "Collections" }).first().click();
      await expect(page.locator("#collections")).toHaveCount(1);
      continue;
    }

    if (check === "deep_link_surface") {
      const res = await page.goto("/product/sample-slug", { waitUntil: "domcontentloaded" });
      expect(ROUTE_STATUS_OK.has(res?.status() ?? 0)).toBeTruthy();
      continue;
    }

    if (check === "search_surface") {
      await goHome(page);
      await page.getByRole("button", { name: "Search" }).click();
      await expect(page.getByPlaceholder("Search")).toBeVisible();
      continue;
    }

    if (check === "product_detail_surface") {
      const mode = await openPdpForJourney(row, page);
      expect(mode, `${row.id} expected PDP to open (direct URL or listing)`).not.toBeNull();
      continue;
    }

    if (check === "quantity_surface") {
      const mode = await openPdpForJourney(row, page);
      expect(mode, `${row.id} expected PDP for quantity controls`).not.toBeNull();
      await page.getByRole("button", { name: "Increase quantity" }).click();
      await page.getByRole("button", { name: "Decrease quantity" }).click();
      continue;
    }

    if (check === "image_gallery_surface") {
      const mode = await openPdpForJourney(row, page);
      expect(mode, `${row.id} expected PDP for gallery interaction`).not.toBeNull();
      const galleryButton = page.locator('button[aria-label^="View image "]').first();
      if ((await galleryButton.count()) > 0) await galleryButton.click();
      await expect(page.locator("body")).toBeVisible();
      continue;
    }

    if (check === "related_products_surface") {
      const mode = await openPdpForJourney(row, page);
      expect(mode, `${row.id} expected PDP for related-products flow`).not.toBeNull();
      await expect(page.locator("body")).toBeVisible();
      continue;
    }

    if (check === "bag_surface") {
      await page.goto("/bag", { waitUntil: "domcontentloaded" });
      await expect(page.locator("body")).toContainText(/bag|cart/i);
      continue;
    }

    if (check === "checkout_surface") {
      await page.goto("/checkout/address", { waitUntil: "domcontentloaded" });
      await expect(page.locator("body")).toContainText(/checkout|sign in/i);
      continue;
    }

    if (check === "profile_surface") {
      await page.goto("/profile", { waitUntil: "domcontentloaded" });
      await expect(page.locator("body")).toContainText(/profile|sign in/i);
      continue;
    }

    if (check === "wishlist_surface") {
      await page.goto("/wishlist", { waitUntil: "domcontentloaded" });
      await expect(page.locator("body")).toContainText(/wishlist|sign in/i);
      continue;
    }

    if (check === "orders_surface") {
      await page.goto("/profile", { waitUntil: "domcontentloaded" });
      await expect(page.locator("body")).toContainText(/orders|sign in/i);
      continue;
    }

    if (check === "refund_surface") {
      await page.goto("/profile", { waitUntil: "domcontentloaded" });
      await expect(page.locator("body")).toContainText(/refund/i);
      continue;
    }

    if (check === "auth_signin_surface") {
      await goHome(page);
      const signInBtn = page.getByRole("button", { name: /sign in/i });
      if ((await signInBtn.count()) > 0) {
        await expect(signInBtn.first()).toBeVisible();
      } else {
        // Mobile header may hide auth CTA and expose menu first.
        await expect(page.getByRole("button", { name: /open menu/i })).toBeVisible();
      }
      continue;
    }

    if (check === "guest_capabilities") {
      const caps = await getWithSeed(request, "/api/auth/capabilities", seed);
      await assertApiEnvelope(caps, `${row.id} guest-capabilities`);
      const body = await caps.json();
      expect(typeof body?.data?.mode).toBe("string");
      continue;
    }

    if (check === "admin_guard_surface") {
      const res = await page.goto("/imtheboss/orders", { waitUntil: "domcontentloaded" });
      expect(res?.status()).toBe(200);
      await expect(page).toHaveURL(/imtheboss\/login|imtheboss\/orders/);
      continue;
    }

    if (check === "payment_verify_surface") {
      const res = await postWithSeed(request, "/api/checkout/verify-payment", seed, {});
      expect([400, 401, 403].includes(res.status())).toBeTruthy();
      continue;
    }

    if (check === "telemetry_surface") {
      const res = await getWithSeed(request, "/api/telemetry/summary", seed);
      expect([200, 401].includes(res.status())).toBeTruthy();
      continue;
    }

    if (check === "not_found_surface") {
      const res = await page.goto("/__journey_missing__", { waitUntil: "domcontentloaded" });
      expect(res?.status()).toBe(404);
      continue;
    }

    if (check === "device_surface") {
      await page.setViewportSize({ width: 375, height: 812 });
      await goHome(page);
      await expect(page.locator("body")).toBeVisible();
      await page.setViewportSize({ width: 1280, height: 720 });
    }
  }
}

async function runScenario(row: JourneyRow, page: Page, request: APIRequestContext) {
  const label = `${row.id} ${row.journey}`;
  const seed = journeySeed(row.id);
  const paths = extractPaths(`${row.sitePath} ${row.endpoint}`);
  expect(paths.length, `${row.id} has no route in Site Path`).toBeGreaterThan(0);
  const behaviorChecks = collectBehaviorChecks(row);
  if (REQUIRE_BEHAVIOR_ASSERTIONS) {
    expect(behaviorChecks.length, `${row.id} has no behavior assertions mapped`).toBeGreaterThan(0);
  }

  const journey = row.journey.toLowerCase();

  if (journey.includes("back/forward navigation")) {
    await assertRoutePath("/", page, label);
    await assertRoutePath("/bag", page, label);
    await page.goBack({ waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/$/);
    await page.goForward({ waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/\/bag$/);
    return;
  }

  if (journey.includes("open non-existent route")) {
    const res = await page.goto("/__journey_missing__", { waitUntil: "domcontentloaded" });
    expect(res?.status()).toBe(404);
    return;
  }

  if (journey.includes("redirect") || journey.includes("blocked from admin routes") || row.id === "UJ-209") {
    const res = await page.goto("/imtheboss/orders", { waitUntil: "domcontentloaded" });
    expect(res?.status()).toBe(200);
    await expect(page).toHaveURL(/imtheboss\/login/);
    await expect(page.getByRole("button", { name: "Sign in with Google" })).toBeVisible();
    return;
  }

  if (journey.includes("session mint") || journey.includes("session creation")) {
    const res = await getWithSeed(request, "/api/auth/capabilities", seed);
    await assertApiEnvelope(res, `${label} /api/auth/capabilities`);
    const body = await res.json();
    expect(body?.ok).toBeTruthy();
    expect(typeof body?.data?.mode).toBe("string");
    return;
  }

  for (const p of paths) {
    if (p.startsWith("/api/")) {
      const res = await getWithSeed(request, p, seed);
      await assertApiEnvelope(res, `${label} ${p}`);
    } else {
      await assertRoutePath(p, page, label);
    }
  }

  await runBehaviorChecks(row, behaviorChecks, page, request, seed);

  if (row.section === 1) {
    await goHome(page);
    if (journey.includes("top nav")) {
      await page.getByRole("button", { name: "Explore" }).first().click();
      await expect(page.locator("#explore")).toHaveCount(1);
    } else if (journey.includes("footer")) {
      await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
      await page.getByRole("button", { name: "Collections" }).first().click();
      await expect(page.locator("#collections")).toHaveCount(1);
    }
    return;
  }

  if (row.section === 2) {
    await goHome(page);
    await page.getByRole("button", { name: "Search" }).click();
    const searchInput = page.getByPlaceholder("Search");
    await expect(searchInput).toBeVisible();
    await searchInput.fill("silk");
    if (journey.includes("filter") || journey.includes("sort")) {
      await page.locator("#explore-collection").selectOption({ index: 0 });
      await page.locator("#explore-sort").selectOption("Latest");
    }
    return;
  }

  if (row.section === 3) {
    const opened = await tryOpenProductFromHome(page);
    if (!opened) return;
    await page.getByRole("button", { name: "Increase quantity" }).click();
    await page.getByRole("button", { name: "Decrease quantity" }).click();
    if (journey.includes("image gallery")) {
      const galleryButton = page.locator('button[aria-label^="View image "]').first();
      if ((await galleryButton.count()) > 0) await galleryButton.click();
    }
    return;
  }

  if (row.section === 5) {
    await page.goto("/bag", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toContainText(/bag/i);
    if (journey.includes("validation error")) {
      const bad = await postWithSeed(request, "/api/account/cart", seed, {
        cartId: "",
        variantId: "",
        quantity: 0,
      });
      expect([400, 401].includes(bad.status())).toBeTruthy();
    }
    return;
  }

  if (row.section === 6) {
    await page.goto("/checkout/address", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toContainText(/checkout|sign in to continue/i);
    return;
  }

  if (row.section === 7) {
    const res = await postWithSeed(request, "/api/checkout/verify-payment", seed, {});
    expect([400, 401, 403].includes(res.status())).toBeTruthy();
    return;
  }

  if (row.section === 8) {
    const caps = await getWithSeed(request, "/api/auth/capabilities", seed);
    await assertApiEnvelope(caps, `${label} capabilities`);
    const profile = await getWithSeed(request, "/api/account/profile", seed);
    expect([200, 401].includes(profile.status())).toBeTruthy();
    return;
  }

  if (row.section === 9) {
    await page.goto("/profile", { waitUntil: "domcontentloaded" });
    await expect(page.locator("body")).toContainText(/profile|sign in/i);
    return;
  }

  if (row.section === 10) {
    await page.goto("/imtheboss/orders", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/imtheboss\/login|imtheboss\/orders/);
    if (page.url().includes("/imtheboss/login")) {
      await expect(page.getByRole("button", { name: "Sign in with Google" })).toBeVisible();
    }
    if (row.id === "UJ-210" || row.id === "UJ-211") {
      const telem = await getWithSeed(request, "/api/telemetry/summary", seed);
      expect([200, 401].includes(telem.status())).toBeTruthy();
    }
    return;
  }

  if (row.section === 11) {
    await runSection11Scenario(journey, request, seed);
    return;
  }

  if (row.section === 12) {
    const ready = await getWithSeed(request, "/api/auth/capabilities", seed);
    await assertApiEnvelope(ready, `${label} resilience-check`);
    return;
  }

  if (row.section === 13) {
    const started = Date.now();
    await goHome(page);
    expect(Date.now() - started).toBeLessThan(10_000);
    return;
  }

  if (row.section === 14) {
    await goHome(page);
    await page.keyboard.press("Tab");
    const focusedTag = await page.evaluate(() => document.activeElement?.tagName ?? "");
    expect(focusedTag.length).toBeGreaterThan(0);
    return;
  }

  if (row.section === 15) {
    await runSection15Scenario(journey, request, seed);
    return;
  }

  if (row.section === 16) {
    await runSection16Scenario(page);
    return;
  }

}

const rows = parseRows();
const sectionFilter = parseSectionFilter();
const providerSafeRows = INCLUDE_PROVIDER_LIVE_JOURNEYS
  ? rows
  : rows.filter((row) => !isProviderLiveRow(row));
const activeRows = sectionFilter
  ? providerSafeRows.filter((r) => r.section !== undefined && sectionFilter.has(r.section))
  : providerSafeRows;

const coreRows = rows.filter((r) => r.id.startsWith("UJ-"));
const extendedRows = rows.filter((r) => r.id.startsWith("UJX-"));

test("journey catalogs exist and matrix rows parse", async () => {
  const files = resolveJourneyFiles();
  expect(files.length).toBeGreaterThan(0);
  expect(coreRows.length).toBeGreaterThanOrEqual(220);
  expect(extendedRows.length).toBeGreaterThanOrEqual(5000);
  expect(activeRows.length).toBeGreaterThan(0);
  if (!INCLUDE_PROVIDER_LIVE_JOURNEYS) {
    const leaked = activeRows.find((row) => isProviderLiveRow(row));
    expect(leaked, "provider-live journeys must be excluded by default CI").toBeUndefined();
  }
});

test("default catalogs block live-provider journey leakage", async () => {
  const findings = DEFAULT_BLUEPRINTS.flatMap((filePath) =>
    collectForbiddenCatalogMatches(filePath)
  );
  const summary = findings
    .map((f) => `${f.file}:${f.line} (${f.pattern}) -> ${f.text}`)
    .join("\n");
  expect(findings, summary || "No forbidden live-provider terms found.").toEqual([]);
});

for (const row of activeRows) {
  const title = `${row.id} scenario`;
  if (DEFERRED_IDS.has(row.id)) {
    test.skip(title, async () => {});
    continue;
  }

  test(title, async ({ page, request }) => {
    await runScenario(row, page, request);
  });
}
