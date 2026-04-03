import "server-only";

import { mkdir, appendFile, readFile } from "fs/promises";
import path from "path";

export type StoredTelemetryEvent = {
  occurredAt: string;
  route: string;
  pageRoute: string | null;
  userMode: "public" | "account" | "admin";
  action: string;
  outcome: "success" | "failure";
  errorClass:
    | "none"
    | "unauthorized"
    | "validation"
    | "retryable"
    | "network"
    | "fatal"
    | "boundary";
  errorCode: string | null;
  message: string | null;
  status: number | null;
  requestId: string | null;
  online: boolean | null;
  userAgent: string | null;
  effectiveType: string | null;
  downlink: number | null;
  rtt: number | null;
};

const TELEMETRY_DIR = path.join(process.cwd(), ".telemetry");
const TELEMETRY_FILE = path.join(TELEMETRY_DIR, "client-events.ndjson");

async function ensureTelemetryFile(): Promise<void> {
  await mkdir(TELEMETRY_DIR, { recursive: true });
}

export async function appendTelemetryEvent(event: StoredTelemetryEvent): Promise<void> {
  await ensureTelemetryFile();
  await appendFile(TELEMETRY_FILE, `${JSON.stringify(event)}\n`, "utf8");
}

function parseNdjson(content: string): StoredTelemetryEvent[] {
  const out: StoredTelemetryEvent[] = [];
  const lines = content.split(/\r?\n/);
  for (const line of lines) {
    const t = line.trim();
    if (!t) continue;
    try {
      const parsed = JSON.parse(t) as StoredTelemetryEvent;
      out.push(parsed);
    } catch {
      // Skip malformed telemetry lines.
    }
  }
  return out;
}

export async function listTelemetryEvents(params: {
  limit: number;
  userMode?: "public" | "account" | "admin";
  outcome?: "success" | "failure";
  errorClass?:
    | "none"
    | "unauthorized"
    | "validation"
    | "retryable"
    | "network"
    | "fatal"
    | "boundary";
  errorCode?: string;
  routeContains?: string;
}): Promise<StoredTelemetryEvent[]> {
  await ensureTelemetryFile();
  let content = "";
  try {
    content = await readFile(TELEMETRY_FILE, "utf8");
  } catch {
    return [];
  }
  let rows = parseNdjson(content);

  if (params.userMode) rows = rows.filter((r) => r.userMode === params.userMode);
  if (params.outcome) rows = rows.filter((r) => r.outcome === params.outcome);
  if (params.errorClass) rows = rows.filter((r) => r.errorClass === params.errorClass);
  if (params.errorCode) rows = rows.filter((r) => (r.errorCode ?? "") === params.errorCode);
  if (params.routeContains) {
    const q = params.routeContains.toLowerCase();
    rows = rows.filter(
      (r) =>
        r.route.toLowerCase().includes(q) ||
        (r.pageRoute ?? "").toLowerCase().includes(q) ||
        r.action.toLowerCase().includes(q)
    );
  }

  rows.sort((a, b) => Date.parse(b.occurredAt) - Date.parse(a.occurredAt));
  return rows.slice(0, Math.max(1, Math.min(params.limit, 500)));
}
