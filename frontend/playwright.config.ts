import { defineConfig, devices } from "@playwright/test";

const port = Number(process.env.PORT ?? 3000);
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? `http://127.0.0.1:${port}`;
const desktopMode = process.env.PW_DESKTOP === "1";

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 30_000,
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 2 : undefined,
  use: {
    baseURL,
    headless: true,
    trace: "retain-on-failure",
    ...(desktopMode
      ? {
          viewport: { width: 1280, height: 720 },
        }
      : {
          ...devices["iPhone 13"],
        }),
  },
  webServer: {
    command: `npm run start -- --hostname 127.0.0.1 --port ${port}`,
    port,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
