#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");

function readFile(relativePath) {
  const fullPath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(fullPath)) {
    throw new Error(`Missing required file: ${relativePath}`);
  }
  return fs.readFileSync(fullPath, "utf8");
}

function assertContains(content, needle, label, failures) {
  if (!content.includes(needle)) {
    failures.push(`${label}: missing "${needle}"`);
  }
}

const failures = [];

try {
  const contractDocPath = "docs/CROSS_LAYER_CONTRACT.md";
  const routeStatesDocPath = "docs/FRONTEND_ROUTE_STATES.md";
  const releaseGatesDocPath = "docs/RELEASE_GATES.md";

  const contractDoc = readFile(contractDocPath);
  const routeStatesDoc = readFile(routeStatesDocPath);
  const releaseGatesDoc = readFile(releaseGatesDocPath);

  assertContains(contractDoc, "## Route families and auth expectations", contractDocPath, failures);
  assertContains(contractDoc, "## Header ownership contract", contractDocPath, failures);
  assertContains(contractDoc, "## Error response shape consumed by frontend", contractDocPath, failures);
  assertContains(contractDoc, "## Idempotency requirements for mutations", contractDocPath, failures);
  assertContains(contractDoc, "## Major mutation contract matrix", contractDocPath, failures);
  assertContains(contractDoc, "## Checkout and payment state mapping", contractDocPath, failures);

  assertContains(routeStatesDoc, "## Public pages", routeStatesDocPath, failures);
  assertContains(routeStatesDoc, "## Account pages", routeStatesDocPath, failures);
  assertContains(routeStatesDoc, "## Admin pages", routeStatesDocPath, failures);
  assertContains(routeStatesDoc, "## Degraded-state UX actions", routeStatesDocPath, failures);

  assertContains(releaseGatesDoc, "scripts/check-validation-parity.sh", releaseGatesDocPath, failures);
  assertContains(
    releaseGatesDoc,
    "scripts/check-frontend-performance-budgets.mjs",
    releaseGatesDocPath,
    failures
  );

  if (failures.length > 0) {
    console.error("Contract/documentation discipline gate failed:");
    for (const failure of failures) {
      console.error(`- ${failure}`);
    }
    process.exit(1);
  }

  console.log("Contract/documentation discipline gate passed.");
} catch (error) {
  console.error(`Failed to run contract/documentation discipline gate: ${error.message}`);
  process.exit(1);
}
