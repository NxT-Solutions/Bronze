import { defineConfig } from "@playwright/test";

// Baselines come from the jammy image in pin.mjs. reducedMotion,
// deviceScaleFactor 1, and animations: "disabled" keep enter transitions
// and retina scaling out of the diff.
export default defineConfig({
  testDir: ".",
  testMatch: "chrome.visual.mjs",
  outputDir: "test-results",
  snapshotPathTemplate: "{testDir}/{testFilePath}-snapshots/{arg}{ext}",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 60_000,
  forbidOnly: Boolean(process.env.CI),
  reporter: process.env.CI ? "line" : "list",
  webServer: {
    command: "node serve.mjs",
    port: 4173,
    reuseExistingServer: false,
    timeout: 15_000,
  },
  use: {
    baseURL: "http://127.0.0.1:4173",
    viewport: { width: 640, height: 720 },
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
    colorScheme: "light",
    locale: "en-US",
    timezoneId: "UTC",
  },
  expect: {
    timeout: 15_000,
    toHaveScreenshot: {
      animations: "disabled",
      caret: "hide",
      scale: "css",
      maxDiffPixelRatio: 0.01,
      threshold: 0.2,
    },
  },
});
