import { defineConfig, devices } from "@playwright/test";

const port = Number(process.env.PORT ?? 3000);
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? `http://127.0.0.1:${port}`;
const desktopMode = process.env.PW_DESKTOP === "1";
const ci = Boolean(process.env.CI);
const disableArtifacts = process.env.PW_DISABLE_ARTIFACTS === "1";
const reuseExistingServer = process.env.PLAYWRIGHT_REUSE_EXISTING_SERVER === "1" || !ci;
const e2eAuthSecret =
  process.env.NEXTAUTH_SECRET ??
  process.env.AUTH_SECRET ??
  "ci-test-nextauth-secret-at-least-32-chars";

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 45_000,
  retries: ci ? 1 : 0,
  workers: ci ? 4 : undefined,
  reporter: ci ? [["blob"], ["line"]] : [["html"], ["line"]],
  outputDir: "test-results",
  use: {
    baseURL,
    headless: true,
    trace: disableArtifacts ? "off" : "retain-on-failure",
    screenshot: disableArtifacts ? "off" : "only-on-failure",
    video: disableArtifacts ? "off" : "retain-on-failure",
  },
  projects: [
    {
      name: desktopMode ? "chromium-desktop" : "chromium-mobile",
      use: desktopMode
        ? {
            browserName: "chromium",
            viewport: { width: 1280, height: 720 },
          }
        : {
            browserName: "chromium",
            ...devices["iPhone 13"],
          },
    },
  ],
  webServer: {
    command: `npm run start -- --hostname 127.0.0.1 --port ${port}`,
    url: `http://127.0.0.1:${port}`,
    cwd: process.cwd(),
    reuseExistingServer,
    timeout: 120_000,
    env: {
      ...process.env,
      NEXTAUTH_SECRET: e2eAuthSecret,
      AUTH_SECRET: e2eAuthSecret,
      NEXTAUTH_URL: process.env.NEXTAUTH_URL ?? `http://127.0.0.1:${port}`,
    },
  },
});
