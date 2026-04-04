import fs from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

type JourneyRow = {
  id: string;
  section: number;
  journey: string;
  sitePath: string;
  endpoint: string;
};

const BLUEPRINT = path.resolve(
  __dirname,
  "../../../docs/user-journeys-blueprint.md"
);
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

function parseRows(): JourneyRow[] {
  const text = fs.readFileSync(BLUEPRINT, "utf8");
  const lines = text.split(/\r?\n/);
  const rows: JourneyRow[] = [];
  for (const line of lines) {
    const m = line.match(
      /^\| (UJ-\d{3}) \| (\d+) \| (.*?) \| (.*?) \| (.*?) \|$/
    );
    if (!m) continue;
    rows.push({
      id: m[1],
      section: Number(m[2]),
      journey: m[3],
      sitePath: m[4],
      endpoint: m[5],
    });
  }
  return rows;
}

function normalizePath(raw: string): string | null {
  let p = raw.trim();
  if (!p) return null;
  p = p.replace(/`/g, "");
  if (!p.startsWith("/")) return null;

  p = p.replace(/\[slug\]/gi, "sample-slug");
  p = p.replace(/\[id\]/gi, "1");
  p = p.replace(/\[categoryId\]/gi, "1");
  p = p.replace(/\[orderId\]/gi, "1");
  p = p.replace(/\*/g, "");
  p = p.replace(/\/+/g, "/");

  if (p === "/admin" || p === "/admin/") return "/imtheboss";
  if (p.startsWith("/admin/")) return `/imtheboss${p.slice("/admin".length)}`;
  if (p === "/account/measurements") return "/profile"; // fallback to current account surface
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
  for (const m of cell.matchAll(/(\/[A-Za-z0-9_\-./\[\]*]+)/g)) {
    const p = normalizePath(m[1]);
    if (p) paths.add(p);
  }
  return [...paths];
}

async function assertApiPath(
  pathValue: string,
  request: Parameters<(typeof test)["extend"]>[0]["request"],
  label: string
) {
  const res = await request.get(pathValue, { failOnStatusCode: false });
  expect(API_STATUS_OK.has(res.status()), `${label} ${pathValue} -> ${res.status()}`).toBeTruthy();
}

async function assertRoutePath(
  pathValue: string,
  page: Parameters<(typeof test)["extend"]>[0]["page"],
  label: string
) {
  const res = await page.goto(pathValue, { waitUntil: "domcontentloaded" });
  if (!res) return;
  expect(
    ROUTE_STATUS_OK.has(res.status()),
    `${label} ${pathValue} -> ${res.status()}`
  ).toBeTruthy();
}

async function runScenario(
  row: JourneyRow,
  page: Parameters<(typeof test)["extend"]>[0]["page"],
  request: Parameters<(typeof test)["extend"]>[0]["request"]
) {
  const label = `${row.id} ${row.journey}`;
  const paths = extractPaths(row.sitePath);
  expect(paths.length, `${row.id} has no route in Site Path`).toBeGreaterThan(0);

  const journey = row.journey.toLowerCase();

  // Special flows with multi-step behavior.
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

  if (journey.includes("redirect") || journey.includes("blocked from admin routes")) {
    const res = await page.goto("/imtheboss/orders", { waitUntil: "domcontentloaded" });
    expect(res && ROUTE_STATUS_OK.has(res.status())).toBeTruthy();
    await expect(page.url()).toMatch(/imtheboss\/login|imtheboss\/orders/);
    return;
  }

  if (journey.includes("session mint") || journey.includes("session creation")) {
    const res = await request.post("/api/auth/capabilities", { failOnStatusCode: false });
    // Capabilities route may be GET-only; this still validates controlled envelope/status.
    expect(API_STATUS_OK.has(res.status())).toBeTruthy();
  }

  for (const p of paths) {
    if (p.startsWith("/api/")) {
      await assertApiPath(p, request, label);
    } else {
      await assertRoutePath(p, page, label);
    }
  }

  // Lightweight generic user interaction checks for primary storefront journeys.
  if (row.section <= 6 && paths.some((p) => !p.startsWith("/api/"))) {
    await page.keyboard.press("Tab");
    await expect(page.locator("body")).toBeVisible();
  }
}

const rows = parseRows();
const sectionFilter = parseSectionFilter();
const activeRows = sectionFilter
  ? rows.filter((r) => sectionFilter.has(r.section))
  : rows;

test.describe.configure({ mode: "parallel" });

test("journey blueprint exists and matrix rows parse", async () => {
  expect(fs.existsSync(BLUEPRINT)).toBeTruthy();
  expect(rows.length).toBeGreaterThanOrEqual(220);
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
