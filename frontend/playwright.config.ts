import { defineConfig, devices } from "@playwright/test";

const port = Number(process.env.PORT ?? 3000);
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? `http://127.0.0.1:${port}`;
const desktopMode = process.env.PW_DESKTOP === "1";
const ci = Boolean(process.env.CI);

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
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
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
    port,
    reuseExistingServer: !ci,
    timeout: 120_000,
  },
});
