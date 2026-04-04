#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const blueprintPath = path.resolve("docs", "user-journeys-blueprint.md");

function fail(message) {
  console.error(`ERROR: ${message}`);
  process.exit(1);
}

if (!fs.existsSync(blueprintPath)) {
  fail(`Blueprint file not found: ${blueprintPath}`);
}

const text = fs.readFileSync(blueprintPath, "utf8");

const rowMatches = [...text.matchAll(/^\| UJ-(\d{3}) \|/gm)];
const rowCount = rowMatches.length;
if (rowCount === 0) {
  fail("No UJ matrix rows found in blueprint.");
}

const ids = new Set(rowMatches.map((m) => Number(m[1])));
for (let id = 196; id <= 220; id += 1) {
  if (!ids.has(id)) {
    fail(`Missing required journey row UJ-${String(id).padStart(3, "0")}.`);
  }
}

const totalLine = text.match(/^Total journey rows in matrix:\s*(\d+)\s*$/m);
if (!totalLine) {
  fail("Missing 'Total journey rows in matrix' line.");
}
const declaredTotal = Number(totalLine[1]);
if (!Number.isFinite(declaredTotal)) {
  fail("Invalid matrix total value.");
}
if (declaredTotal !== rowCount) {
  fail(
    `Matrix total mismatch: declared=${declaredTotal}, detected=${rowCount}.`
  );
}

const deferredIds = [204, 205, 206, 207];
for (const id of deferredIds) {
  const tag = new RegExp(
    `^\\| UJ-${String(id).padStart(3, "0")} \\|.*\\[Deferred - do not implement now\\]`,
    "m"
  );
  if (!tag.test(text)) {
    fail(
      `Expected deferred marker on UJ-${String(id).padStart(3, "0")} row.`
    );
  }
}

console.log(
  `Journey blueprint check passed. rows=${rowCount}, requiredIDs=UJ-196..UJ-220.`
);
