#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");
const frontendDir = path.join(repoRoot, "frontend");
const nextDir = path.join(frontendDir, ".next");

function toKB(bytes) {
  return Number((bytes / 1024).toFixed(2));
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function fileSizeBytes(filePath) {
  return fs.statSync(filePath).size;
}

function checkExists(filePath) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`Missing required build artifact: ${filePath}`);
  }
}

const violations = [];
const checks = [];

function assertBudget(name, actualBytes, maxBytes, detailPath) {
  const ok = actualBytes <= maxBytes;
  checks.push({
    name,
    actualKB: toKB(actualBytes),
    maxKB: toKB(maxBytes),
    detailPath,
    ok,
  });
  if (!ok) {
    violations.push(
      `${name}: ${toKB(actualBytes)}KB exceeds budget ${toKB(maxBytes)}KB (${detailPath})`
    );
  }
}

try {
  checkExists(nextDir);

  const buildManifestPath = path.join(nextDir, "build-manifest.json");
  checkExists(buildManifestPath);
  const buildManifest = readJson(buildManifestPath);

  const rootAssets = [
    ...(buildManifest.polyfillFiles ?? []),
    ...(buildManifest.rootMainFiles ?? []),
  ];
  const uniqueRootAssets = [...new Set(rootAssets)];

  let rootTotalBytes = 0;
  for (const asset of uniqueRootAssets) {
    const fullAssetPath = path.join(nextDir, asset);
    checkExists(fullAssetPath);
    const size = fileSizeBytes(fullAssetPath);
    rootTotalBytes += size;
    assertBudget(
      `root asset ${asset}`,
      size,
      300 * 1024,
      path.relative(repoRoot, fullAssetPath)
    );
  }
  assertBudget(
    "first-load root JS total",
    rootTotalBytes,
    900 * 1024,
    "build-manifest rootMainFiles + polyfillFiles"
  );

  const routeBudgets = [
    { rel: "server/app/page_client-reference-manifest.js", maxKB: 14 },
    { rel: "server/app/product/[id]/page_client-reference-manifest.js", maxKB: 14 },
    { rel: "server/app/collections/[slug]/page_client-reference-manifest.js", maxKB: 15 },
    { rel: "server/app/bag/page_client-reference-manifest.js", maxKB: 13 },
    { rel: "server/app/checkout/address/page_client-reference-manifest.js", maxKB: 13 },
    { rel: "server/app/profile/page_client-reference-manifest.js", maxKB: 13 },
    { rel: "server/app/imtheboss/page_client-reference-manifest.js", maxKB: 20 },
    { rel: "server/app/imtheboss/products/page_client-reference-manifest.js", maxKB: 15 },
    { rel: "server/app/imtheboss/orders/page_client-reference-manifest.js", maxKB: 15 },
    { rel: "server/app/imtheboss/customers/page_client-reference-manifest.js", maxKB: 15 },
  ];

  for (const budget of routeBudgets) {
    const full = path.join(nextDir, budget.rel);
    checkExists(full);
    const size = fileSizeBytes(full);
    assertBudget(
      `route manifest ${budget.rel}`,
      size,
      budget.maxKB * 1024,
      path.relative(repoRoot, full)
    );
  }

  const chunksDir = path.join(nextDir, "static", "chunks");
  checkExists(chunksDir);
  const chunkFiles = fs
    .readdirSync(chunksDir)
    .filter((name) => name.endsWith(".js"))
    .map((name) => path.join(chunksDir, name));
  const largestChunk = chunkFiles
    .map((filePath) => ({ filePath, bytes: fileSizeBytes(filePath) }))
    .sort((a, b) => b.bytes - a.bytes)[0];

  if (largestChunk) {
    assertBudget(
      "largest static chunk",
      largestChunk.bytes,
      9000 * 1024,
      path.relative(repoRoot, largestChunk.filePath)
    );
  }

  for (const check of checks) {
    const status = check.ok ? "OK" : "FAIL";
    console.log(
      `[${status}] ${check.name}: ${check.actualKB}KB / ${check.maxKB}KB (${check.detailPath})`
    );
  }

  if (violations.length > 0) {
    console.error("\nPerformance budget violations:");
    for (const v of violations) {
      console.error(`- ${v}`);
    }
    process.exit(1);
  }

  console.log("\nFrontend performance budgets passed.");
} catch (error) {
  console.error(`Failed to verify frontend performance budgets: ${error.message}`);
  process.exit(1);
}
