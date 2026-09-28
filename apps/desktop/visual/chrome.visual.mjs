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
    await expect(page.locator("#queue-sort-toggle")).toBeVisible();
    await expect(page.locator("#quick-panel")).toHaveScreenshot(
      shot("queue-empty"),
    );
  });

  test("queue populated", async ({ page }) => {
    await openSurface(page, "/index.html", "populated");
    await expect(page.locator("#queue .queue-item")).toHaveCount(2);
    await expect(page.locator("#queue-empty")).toBeHidden();
    await expect(page.locator("#queue-sort-toggle")).toBeVisible();
    await expect(page.locator("#quick-panel")).toHaveScreenshot(
      shot("queue-populated"),
    );
  });

  test("title swap keeps the reserved title box", async ({ page }) => {
    await openSurface(page, "/index.html", "populated");
    const title = page
      .locator("#queue .queue-item [data-slot='title']")
      .first();
    await expect(title).toBeVisible();
    const measured = await page.evaluate(() => {
      const row = document.querySelector("#queue .queue-item");
      const heading = row.querySelector("[data-slot='title']");
      const height = heading.getBoundingClientRect().height;
      heading.textContent =
        "The generated title wraps onto a second line of this queue card";
      const afterLong = heading.getBoundingClientRect().height;
      const sameRow = document.querySelector("#queue .queue-item") === row;
      heading.textContent = "Hi";
      const afterShort = heading.getBoundingClientRect().height;
      return { height, afterLong, afterShort, sameRow };
    });
    expect(measured.height).toBeGreaterThan(0);
    expect(measured.afterLong).toBe(measured.height);
    expect(measured.afterShort).toBe(measured.height);
    expect(measured.sameRow).toBe(true);
  });

  test("collapsed body clip is three line boxes", async ({ page }) => {
    await openSurface(page, "/index.html", "populated");
    const body = page.locator("#queue .queue-item [data-slot='body']").first();
    await expect(body).toBeVisible();
    const measured = await page.evaluate(() => {
      const article = document.querySelector("#queue .queue-item article");
      const preview = article.querySelector("[data-slot='body']");
      const expand = article.querySelector("[data-slot='expand']");
      const lineHeight = Number.parseFloat(
        getComputedStyle(preview).lineHeight,
      );
      const measure = (text) => {
        preview.textContent = text;
        const box = preview.getBoundingClientRect();
        return box.height;
      };
      const three = measure("AAAA\nBBBB\nCCCC");
      const long = measure(
        "AAAA\nBBBB\nCCCC\n________________\n________________",
      );
      expand.hidden = false;
      const box = preview.getBoundingClientRect();
      const expandBox = expand.getBoundingClientRect();
      const gapTop = box.bottom;
      const gapBottom = expandBox.top;
      const midX = box.left + box.width / 2;
      const gapHits = [];
      if (gapBottom - gapTop > 0.5) {
        const steps = 3;
        for (let i = 0; i < steps; i += 1) {
          const y = gapTop + ((i + 0.5) * (gapBottom - gapTop)) / steps;
          const hit = document.elementFromPoint(midX, y);
          gapHits.push(hit?.getAttribute?.("data-slot") ?? hit?.tagName ?? "");
        }
      }
      const style = getComputedStyle(preview);
      return {
        three,
        long,
        lineHeight,
        maxHeight: style.maxHeight,
        overflow: style.overflow,
        gap: gapBottom - gapTop,
        gapHits,
      };
    });
    expect(measured.lineHeight).toBeGreaterThan(0);
    expect(measured.three).toBeCloseTo(3 * measured.lineHeight, 0);
    expect(measured.long).toBeCloseTo(3 * measured.lineHeight, 0);
    expect(measured.long).toBeCloseTo(measured.three, 0);
    expect(measured.overflow).toBe("hidden");
    expect(measured.gap).toBeGreaterThan(0);
    expect(measured.gapHits.every((slot) => slot !== "body")).toBe(true);
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
