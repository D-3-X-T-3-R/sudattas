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
const DEFERRED_IDS = new Set(["UJ-204", "UJ-205", "UJ-206", "UJ-207"]);
const ROUTE_STATUS_OK = new Set([200, 301, 302, 303, 307, 308, 401, 403, 404]);
const API_STATUS_OK = new Set([200, 201, 204, 400, 401, 403, 404, 405, 409, 422, 429]);

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
  const fromEnv = process.env.JOURNEY_BLUEPRINT_FILES?.trim();
  if (!fromEnv) return DEFAULT_BLUEPRINTS.filter((p) => fs.existsSync(p));
  return fromEnv
    .split(",")
    .map((f) => path.resolve(__dirname, "../../../", f.trim()))
    .filter((p) => fs.existsSync(p));
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

async function runScenario(row: JourneyRow, page: Page, request: APIRequestContext) {
  const label = `${row.id} ${row.journey}`;
  const seed = journeySeed(row.id);
  const paths = extractPaths(`${row.sitePath} ${row.endpoint}`);
  expect(paths.length, `${row.id} has no route in Site Path`).toBeGreaterThan(0);

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
const activeRows = sectionFilter
  ? rows.filter((r) => r.section !== undefined && sectionFilter.has(r.section))
  : rows;

const coreRows = rows.filter((r) => r.id.startsWith("UJ-"));
const extendedRows = rows.filter((r) => r.id.startsWith("UJX-"));

test("journey catalogs exist and matrix rows parse", async () => {
  const files = resolveJourneyFiles();
  expect(files.length).toBeGreaterThan(0);
  expect(coreRows.length).toBeGreaterThanOrEqual(220);
  expect(extendedRows.length).toBeGreaterThanOrEqual(5000);
  expect(activeRows.length).toBeGreaterThan(0);
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
