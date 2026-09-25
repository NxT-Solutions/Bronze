import { expect, test } from "@playwright/test";
import { installVisualStub } from "./stub.mjs";

function shot(name) {
  return `${name}-${process.arch}.png`;
}

async function openSurface(page, path, scenario) {
  await page.addInitScript(installVisualStub, scenario);
  await page.goto(path, { waitUntil: "load" });
  await expect(page.locator("html")).toHaveAttribute("data-motion", "reduce");
  await page.evaluate(async () => {
    await document.fonts.ready;
  });
}

test.describe("queue", () => {
  test.use({ viewport: { width: 400, height: 720 } });

  test("queue empty", async ({ page }) => {
    await openSurface(page, "/index.html", "empty");
    await expect(page.locator("#queue-empty")).toBeVisible();
    await expect(page.locator("#queue .queue-item")).toHaveCount(0);
    await expect(page.locator("#quick-panel")).toHaveScreenshot(
      shot("queue-empty"),
    );
  });

  test("queue populated", async ({ page }) => {
    await openSurface(page, "/index.html", "populated");
    await expect(page.locator("#queue .queue-item")).toHaveCount(2);
    await expect(page.locator("#queue-empty")).toBeHidden();
    await expect(page.locator("#quick-panel")).toHaveScreenshot(
      shot("queue-populated"),
    );
  });
});

test.describe("settings", () => {
  test.use({ viewport: { width: 640, height: 720 } });

  test("settings permission denied", async ({ page }) => {
    await openSurface(page, "/settings.html", "empty");
    const health = page.locator("[data-slot='permission-health']");
    await expect(
      health.locator("[data-capability='accessibility'] .pill"),
    ).toHaveText("Denied");
    await expect(
      health.locator("[data-capability='notifications'] .pill"),
    ).toHaveText("Unavailable");
    await expect(health).toHaveScreenshot(shot("settings-permission-denied"));
  });

  test("outlined actions", async ({ page }) => {
    await openSurface(page, "/settings.html", "empty");
    await expect(page.locator("[data-check-update]")).toHaveText(
      "Check for updates",
    );
    await expect(page.locator("[data-import-title-gguf]")).toHaveText(
      "Import GGUF…",
    );
    await expect(page.locator("[data-check-update]")).toHaveScreenshot(
      shot("outlined-check-updates"),
    );
    await expect(page.locator("[data-import-title-gguf]")).toHaveScreenshot(
      shot("outlined-import-gguf"),
    );
  });
});

test.describe("library", () => {
  test.use({ viewport: { width: 720, height: 640 } });

  test("library empty", async ({ page }) => {
    await openSurface(page, "/library.html", "empty");
    await expect(page.locator("#library-empty-title")).toBeVisible();
    await expect(page.locator("#library-queue .queue-item")).toHaveCount(0);
    await expect(page.locator("#library")).toHaveScreenshot(
      shot("library-empty"),
    );
  });

  test("library populated", async ({ page }) => {
    await openSurface(page, "/library.html", "populated");
    await expect(page.locator("#library-queue .queue-item")).toHaveCount(2);
    await expect(page.locator("#library-empty-title")).toBeHidden();
    await expect(page.locator("#library")).toHaveScreenshot(
      shot("library-populated"),
    );
  });
});

test.describe("help", () => {
  test.use({ viewport: { width: 640, height: 640 } });

  test("help", async ({ page }) => {
    await openSurface(page, "/help.html", "empty");
    await expect(page.locator("[data-diagnostics-preview]")).toContainText(
      "generated_at_ms=unavailable",
    );
    await expect(page.locator("#help")).toHaveScreenshot(shot("help"));
  });
});
