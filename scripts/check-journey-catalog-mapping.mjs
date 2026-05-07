#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const defaultCatalogs = [
  path.resolve("docs", "user-journeys-blueprint.md"),
  path.resolve("docs", "user-journeys-catalog-5000.md"),
];
const providerLiveCatalog = path.resolve("docs", "user-journeys-provider-live.md");

const requiredBuckets = [
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
];

const bucketModes = {
  entry_discovery: { mode: "browser" },
  catalog_browsing: { mode: "browser" },
  product_detail: {
    mode: "catalog_only",
    reason:
      "PDP route is server-fetched and requires seeded deterministic product fixtures in this pipeline; covered by catalog mapping + component tests.",
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
      "Refund lifecycle permutations are primarily validated by account API/unit tests and journey catalog validation.",
  },
  wishlist: { mode: "browser" },
  admin_auth: { mode: "browser" },
  admin_catalog: {
    mode: "catalog_only",
    reason:
      "Admin catalog CRUD requires seeded admin datasets and is already covered by admin unit/contract tests.",
  },
  admin_orders: {
    mode: "catalog_only",
    reason:
      "Admin order/refund/dispute operations rely on backend reconciliation datasets; validated in backend integration and catalog checks.",
  },
  admin_customers: {
    mode: "catalog_only",
    reason:
      "Admin customer/audit reporting needs seeded privileged data and is validated by contract-level checks.",
  },
  security_negative: { mode: "browser" },
  reliability_error: { mode: "browser" },
  accessibility_smoke: { mode: "browser" },
  seo_content: {
    mode: "catalog_only",
    reason:
      "SEO/static routes are validated via dedicated sitemap/robots/static-page tests and script checks.",
  },
  responsive_mobile: { mode: "browser" },
};

const knownExtendedDomains = new Set([
  "Admin Audit, Compliance & Reporting",
  "Admin Auth & Access",
  "Admin Catalog & Merchandising",
  "Admin Order Operations",
  "Admin Refunds, Risk & Disputes",
  "User Account & Profile",
  "User Login & Identity",
  "User Orders Tracking",
  "User Refunds & Returns",
  "User Security & Abuse",
  "User Wishlist",
]);

const forbiddenProviderPatterns = [
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

const allowedProviderContext = /\b(mock-safe|excluded from default ci|provider-live\.md)\b/i;

const knownPathPrefixes = [
  "/",
  "/account",
  "/admin",
  "/api",
  "/auth",
  "/bag",
  "/cart",
  "/categories",
  "/category",
  "/checkout",
  "/collections",
  "/does-not-exist",
  "/help/contact",
  "/imtheboss",
  "/login",
  "/product",
  "/products",
  "/profile",
  "/wishlist",
  "/query",
  "/queries",
  "/ready",
  "/robots.txt",
  "/session",
  "/sitemap.xml",
  "/v2",
  "/graphql",
  "/404",
  "/500",
];

const intentionalNarrativePaths = new Set([
  "/api",
  "/graphql",
  "/browser",
  "/degraded",
  "/disallowed",
  "/forward",
  "/grpc",
  "/identity",
  "/invalid",
  "/orders",
  "/needs-review",
  "/or",
  "/protected",
  "/qty",
  "/reconciliation",
  "/recovery",
  "/remove",
  "/runtime",
  "/video",
]);

const args = process.argv.slice(2);
const jsonOnly = args.includes("--json");

function fail(message) {
  if (!jsonOnly) {
    console.error(`ERROR: ${message}`);
  }
  process.exitCode = 1;
  throw new Error(message);
}

function splitMarkdownRow(line) {
  if (!line.startsWith("|")) return null;
  const cells = [];
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

function containsAny(text, needles) {
  return needles.some((n) => text.includes(n));
}

function extractPaths(text) {
  const paths = new Set();
  for (const m of text.matchAll(/`([^`]+)`/g)) {
    const normalized = normalizePathToken(m[1]);
    if (normalized) paths.add(normalized);
  }
  // Match plain-text route tokens while avoiding narrative fragments such as
  // "auth/error" or "queries/mutations" where "/" is not a URL start.
  for (const m of text.matchAll(/(?:^|[\s(,:;+`"'|-])(\/[A-Za-z0-9_\-\[\]/*.|?=]+)/g)) {
    const normalized = normalizePathToken(m[1]);
    if (normalized) paths.add(normalized);
  }
  return [...paths];
}

function normalizePathToken(raw) {
  let value = raw.trim();
  if (!value.startsWith("/")) return null;
  const queryIndex = value.indexOf("?");
  if (queryIndex >= 0) {
    value = value.slice(0, queryIndex);
  }
  value = value.replace(/[),.;]+$/g, "");
  value = value.replace(/\s+/g, "");
  value = value.replace(/<[^>]+>/g, "");
  if (!value) return null;
  return value;
}

function isKnownOrIntentionalPath(pathToken) {
  if (!pathToken) return false;
  const lower = pathToken.toLowerCase();
  if (intentionalNarrativePaths.has(lower)) return true;
  if (knownPathPrefixes.some((prefix) => lower === prefix || lower.startsWith(`${prefix}/`))) {
    return true;
  }
  return false;
}

function mapBlueprintRowToBucket(row) {
  const text = `${row.journey} ${row.userPath} ${row.endpoint}`.toLowerCase();
  switch (row.section) {
    case 1:
      return "entry_discovery";
    case 2:
      return "catalog_browsing";
    case 3:
      return "product_detail";
    case 4:
      return "guest_session";
    case 5:
      return "cart";
    case 6:
      return "checkout";
    case 7:
      return "payment_states";
    case 8:
      return "auth";
    case 9:
      if (text.includes("wishlist")) return "wishlist";
      if (containsAny(text, ["refund", "return", "cancel"])) return "account_refunds";
      return "account_orders";
    case 10:
      if (containsAny(text, ["admin login", "forged admin", "blocked from admin", "admin auth", "admin route access", "unauthorized"])) {
        return "admin_auth";
      }
      if (containsAny(text, ["customer", "users", "user management"])) return "admin_customers";
      if (containsAny(text, ["product", "catalog", "category", "inventory", "coupon", "review", "lookbook", "merchandising"])) {
        return "admin_catalog";
      }
      if (containsAny(text, ["order", "shipment", "refund", "transaction", "risk", "dispute"])) {
        return "admin_orders";
      }
      return "admin_orders";
    case 11:
      return "security_negative";
    case 12:
    case 13:
      return "reliability_error";
    case 14:
      return "accessibility_smoke";
    case 15:
      return "seo_content";
    case 16:
      return "responsive_mobile";
    default:
      return null;
  }
}

function mapExtendedRowToBucket(row) {
  const text = `${row.journey} ${row.userPath} ${row.expected}`.toLowerCase();
  switch (row.domain) {
    case "User Login & Identity":
      return "auth";
    case "User Account & Profile":
      if (text.includes("wishlist")) return "wishlist";
      if (containsAny(text, ["refund", "return", "cancel"])) return "account_refunds";
      return "account_orders";
    case "User Orders Tracking":
      return "account_orders";
    case "User Refunds & Returns":
      return "account_refunds";
    case "User Wishlist":
      return "wishlist";
    case "User Security & Abuse":
      return "security_negative";
    case "Admin Auth & Access":
      return "admin_auth";
    case "Admin Catalog & Merchandising":
      return "admin_catalog";
    case "Admin Order Operations":
      return "admin_orders";
    case "Admin Refunds, Risk & Disputes":
      return "admin_orders";
    case "Admin Audit, Compliance & Reporting":
      return "admin_customers";
    default:
      return null;
  }
}

function parseBlueprintRows(filePath) {
  const text = fs.readFileSync(filePath, "utf8");
  if (!/\|\s*ID\s*\|\s*Section\s*\|\s*Journey\s*\|\s*Site Path User Takes\s*\|\s*Primary API\/GraphQL Endpoint\(s\)\s*\|/i.test(text)) {
    fail(`Blueprint header columns missing in ${path.basename(filePath)}.`);
  }

  const rows = [];
  const lines = text.split(/\r?\n/);
  for (const line of lines) {
    const cells = splitMarkdownRow(line);
    if (!cells || cells.length < 5) continue;
    const id = cells[0];
    if (!/^UJ-\d{3}$/.test(id)) continue;
    const section = Number(cells[1]);
    if (!Number.isFinite(section) || section < 1 || section > 16) {
      fail(`Invalid section on ${id} in ${path.basename(filePath)}.`);
    }
    rows.push({
      id,
      source: path.basename(filePath),
      type: "blueprint",
      section,
      journey: cells[2] ?? "",
      userPath: cells[3] ?? "",
      endpoint: cells[4] ?? "",
      domain: null,
      expected: null,
    });
  }
  return rows;
}

function parseExtendedRows(filePath) {
  const text = fs.readFileSync(filePath, "utf8");
  if (!/\|\s*ID\s*\|\s*Domain\s*\|\s*Journey\s*\|\s*User Path\s*\|\s*Expected Result\s*\|/i.test(text)) {
    fail(`Extended catalog header columns missing in ${path.basename(filePath)}.`);
  }

  const rows = [];
  const lines = text.split(/\r?\n/);
  for (const line of lines) {
    const cells = splitMarkdownRow(line);
    if (!cells || cells.length < 5) continue;
    const id = cells[0];
    if (!/^UJX-\d{4}$/.test(id)) continue;
    const domain = cells[1] ?? "";
    if (!knownExtendedDomains.has(domain)) {
      fail(`Unknown extended domain '${domain}' on ${id}.`);
    }
    rows.push({
      id,
      source: path.basename(filePath),
      type: "extended",
      section: null,
      journey: cells[2] ?? "",
      userPath: cells[3] ?? "",
      endpoint: "",
      domain,
      expected: cells[4] ?? "",
    });
  }
  return rows;
}

function collectProviderLeakFindings(filePaths) {
  const findings = [];
  for (const filePath of filePaths) {
    const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
    lines.forEach((line, index) => {
      for (const pattern of forbiddenProviderPatterns) {
        if (!pattern.test(line)) continue;
        if (allowedProviderContext.test(line)) continue;
        findings.push({
          file: path.basename(filePath),
          line: index + 1,
          pattern: pattern.toString(),
          text: line.trim(),
        });
      }
    });
  }
  return findings;
}

function ensureProviderLiveCatalogPolicy(filePath) {
  if (!fs.existsSync(filePath)) {
    fail(`Missing provider-live catalog: ${filePath}`);
  }
  const text = fs.readFileSync(filePath, "utf8");
  if (!text.includes("RUN_LIVE_PROVIDER_JOURNEYS=1")) {
    fail("Provider-live catalog missing RUN_LIVE_PROVIDER_JOURNEYS gate documentation.");
  }
  if (!text.includes("PROVIDER_LIVE_TEST_CONFIRM=I_UNDERSTAND_THIS_HITS_REAL_PROVIDERS")) {
    fail("Provider-live catalog missing PROVIDER_LIVE_TEST_CONFIRM gate documentation.");
  }
}

function parseProviderLiveCount(filePath) {
  const text = fs.readFileSync(filePath, "utf8");
  return [...text.matchAll(/^\|\s*UJPL-\d{3}\s*\|/gm)].length;
}

function main() {
  const missing = defaultCatalogs.filter((filePath) => !fs.existsSync(filePath));
  if (missing.length > 0) {
    fail(`Missing default catalogs: ${missing.join(", ")}`);
  }

  ensureProviderLiveCatalogPolicy(providerLiveCatalog);

  const blueprintRows = parseBlueprintRows(defaultCatalogs[0]);
  const extendedRows = parseExtendedRows(defaultCatalogs[1]);
  const rows = [...blueprintRows, ...extendedRows];

  if (blueprintRows.length < 220) {
    fail(`Expected at least 220 blueprint rows, found ${blueprintRows.length}.`);
  }
  if (extendedRows.length < 5000) {
    fail(`Expected at least 5000 extended rows, found ${extendedRows.length}.`);
  }

  const ids = new Set();
  const duplicateIds = [];
  for (const row of rows) {
    if (ids.has(row.id)) duplicateIds.push(row.id);
    ids.add(row.id);
  }
  if (duplicateIds.length > 0) {
    fail(`Duplicate journey IDs detected: ${[...new Set(duplicateIds)].join(", ")}`);
  }

  const providerLeakFindings = collectProviderLeakFindings(defaultCatalogs);
  if (providerLeakFindings.length > 0) {
    const details = providerLeakFindings
      .map((f) => `${f.file}:${f.line} (${f.pattern}) -> ${f.text}`)
      .join("\n");
    fail(`Provider-live leakage found in default catalogs:\n${details}`);
  }

  const bucketCounts = Object.fromEntries(requiredBuckets.map((bucket) => [bucket, 0]));
  let mappedCount = 0;
  let unknownUnmappedCount = 0;
  const unmappedRows = [];
  const unknownPathFindings = [];
  const idToBucket = {};

  for (const row of rows) {
    const bucket = row.type === "blueprint" ? mapBlueprintRowToBucket(row) : mapExtendedRowToBucket(row);
    if (!bucket || !requiredBuckets.includes(bucket)) {
      unknownUnmappedCount += 1;
      unmappedRows.push({ id: row.id, source: row.source, journey: row.journey });
    } else {
      mappedCount += 1;
      bucketCounts[bucket] += 1;
      idToBucket[row.id] = bucket;
    }

    const pathTokens = extractPaths(`${row.userPath} ${row.endpoint} ${row.expected ?? ""}`);
    for (const token of pathTokens) {
      if (!isKnownOrIntentionalPath(token)) {
        unknownPathFindings.push({ id: row.id, source: row.source, token });
      }
    }
  }

  if (unknownPathFindings.length > 0) {
    const sample = unknownPathFindings
      .slice(0, 20)
      .map((f) => `${f.id} (${f.source}) -> ${f.token}`)
      .join("\n");
    fail(`Unknown path tokens found (sample):\n${sample}`);
  }

  if (unknownUnmappedCount > 0) {
    const sample = unmappedRows.slice(0, 20).map((r) => `${r.id} (${r.source})`).join("\n");
    fail(`Unmapped journey rows found (${unknownUnmappedCount}):\n${sample}`);
  }

  const missingBucketConfig = requiredBuckets.filter((bucket) => !Object.hasOwn(bucketModes, bucket));
  if (missingBucketConfig.length > 0) {
    fail(`Bucket mode configuration missing for: ${missingBucketConfig.join(", ")}`);
  }

  for (const bucket of requiredBuckets) {
    if (bucketModes[bucket].mode === "catalog_only" && !bucketModes[bucket].reason) {
      fail(`Catalog-only bucket '${bucket}' is missing reason.`);
    }
  }

  const catalogOnlyCount = requiredBuckets
    .filter((bucket) => bucketModes[bucket].mode === "catalog_only")
    .reduce((sum, bucket) => sum + bucketCounts[bucket], 0);
  const browserRepresentedCount = requiredBuckets
    .filter((bucket) => bucketModes[bucket].mode === "browser")
    .reduce((sum, bucket) => sum + bucketCounts[bucket], 0);

  const providerLiveExcluded = parseProviderLiveCount(providerLiveCatalog);

  const report = {
    totals: {
      totalJourneysParsed: rows.length,
      blueprintRows: blueprintRows.length,
      extendedRows: extendedRows.length,
      mappedCount,
      catalogOnlyCount,
      browserRepresentedCount,
      providerLiveExcluded,
      unknownUnmappedCount,
      providerLeakFindings: providerLeakFindings.length,
      unknownPathFindings: unknownPathFindings.length,
    },
    bucketCounts,
    bucketModes,
    requiredBuckets,
    sampleIdsByBucket: Object.fromEntries(
      requiredBuckets.map((bucket) => [
        bucket,
        rows.filter((row) => idToBucket[row.id] === bucket).slice(0, 5).map((row) => row.id),
      ])
    ),
    idToBucket,
  };

  if (jsonOnly) {
    process.stdout.write(`${JSON.stringify(report)}\n`);
    return;
  }

  console.log("Journey catalog validation report:");
  console.log(`- total journeys parsed: ${report.totals.totalJourneysParsed}`);
  console.log(`- blueprint rows: ${report.totals.blueprintRows}`);
  console.log(`- extended rows: ${report.totals.extendedRows}`);
  console.log(`- mapped to buckets: ${report.totals.mappedCount}`);
  console.log(`- catalog-only rows: ${report.totals.catalogOnlyCount}`);
  console.log(`- browser-represented rows: ${report.totals.browserRepresentedCount}`);
  console.log(`- provider-live excluded rows: ${report.totals.providerLiveExcluded}`);
  console.log(`- unknown/unmapped rows: ${report.totals.unknownUnmappedCount}`);
  console.log("- bucket counts:");
  for (const bucket of requiredBuckets) {
    const mode = bucketModes[bucket].mode;
    console.log(`  - ${bucket}: ${bucketCounts[bucket]} (${mode})`);
  }
  console.log("Journey catalog mapping check passed.");
}

try {
  main();
} catch (error) {
  if (jsonOnly) {
    const message = error instanceof Error ? error.message : String(error);
    process.stdout.write(`${JSON.stringify({ ok: false, error: message })}\n`);
  }
  if (process.exitCode === undefined || process.exitCode === 0) {
    process.exit(1);
  } else {
    process.exit(process.exitCode);
  }
}
