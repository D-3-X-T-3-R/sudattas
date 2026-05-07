#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const defaultCatalogs = [
  path.resolve("docs", "user-journeys-blueprint.md"),
  path.resolve("docs", "user-journeys-catalog-5000.md"),
];

const providerLiveCatalog = path.resolve("docs", "user-journeys-provider-live.md");

const forbiddenPatterns = [
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

const allowedInlineContext = /\b(mock-safe|excluded from default ci|provider-live\.md)\b/i;

function fail(message) {
  console.error(`ERROR: ${message}`);
  process.exit(1);
}

const missing = defaultCatalogs.filter((filePath) => !fs.existsSync(filePath));
if (missing.length > 0) {
  fail(`Missing default journey catalogs: ${missing.join(", ")}`);
}

if (!fs.existsSync(providerLiveCatalog)) {
  fail(`Missing provider-live catalog: ${providerLiveCatalog}`);
}

const findings = [];
for (const filePath of defaultCatalogs) {
  const text = fs.readFileSync(filePath, "utf8");
  const lines = text.split(/\r?\n/);
  lines.forEach((line, index) => {
    for (const pattern of forbiddenPatterns) {
      if (!pattern.test(line)) continue;
      if (allowedInlineContext.test(line)) continue;
      findings.push({
        file: path.basename(filePath),
        line: index + 1,
        pattern: pattern.toString(),
        text: line.trim(),
      });
    }
  });
}

if (findings.length > 0) {
  const summary = findings
    .map((f) => `${f.file}:${f.line} (${f.pattern}) -> ${f.text}`)
    .join("\n");
  fail(`Default catalogs contain live-provider leakage:\n${summary}`);
}

console.log(
  "Provider-safe journey catalog check passed. Default catalogs are free from live-provider journey terms."
);
