import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { PLAYWRIGHT_IMAGE, PLAYWRIGHT_VERSION } from "./pin.mjs";
import config from "./playwright.config.mjs";
import { installVisualStub } from "./stub.mjs";

const repoRoot = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "..",
);

test("playwright config keeps the screenshot flake controls", () => {
  assert.equal(config.use.deviceScaleFactor, 1);
  assert.equal(config.use.reducedMotion, "reduce");
  assert.equal(config.use.viewport.width, 640);
  assert.equal(config.expect.toHaveScreenshot.animations, "disabled");
  assert.equal(config.expect.toHaveScreenshot.caret, "hide");
  assert.equal(config.expect.toHaveScreenshot.scale, "css");
  assert.equal(config.expect.toHaveScreenshot.maxDiffPixelRatio, 0.01);
  assert.equal(config.expect.toHaveScreenshot.threshold, 0.2);
  assert.equal(config.workers, 1);
  assert.equal(config.retries, 0);
});

test("playwright package matches the Linux image that owns the baselines", () => {
  const pkg = JSON.parse(
    readFileSync(join(repoRoot, "apps/desktop/package.json"), "utf8"),
  );
  assert.equal(pkg.devDependencies["@playwright/test"], PLAYWRIGHT_VERSION);
  assert.equal(
    PLAYWRIGHT_IMAGE,
    `mcr.microsoft.com/playwright:v${PLAYWRIGHT_VERSION}-jammy`,
  );
  const runner = readFileSync(join(repoRoot, "tooling/visual-run.mjs"), "utf8");
  assert.match(runner, /PLAYWRIGHT_IMAGE/);
  assert.match(runner, /linux\/arm64/);
  assert.match(runner, /linux\/amd64/);
});

test("CI runs the visual workflow and a miss fails the CI check", () => {
  const ci = readFileSync(join(repoRoot, ".github/workflows/ci.yml"), "utf8");
  const visual = readFileSync(
    join(repoRoot, ".github/workflows/visual.yml"),
    "utf8",
  );
  assert.match(ci, /uses:\s+\.\/\.github\/workflows\/visual\.yml/);
  assert.match(ci, /needs:\s*\[quality-visual\]/);
  assert.match(ci, /name:\s*CI/);
  assert.match(visual, /node tooling\/visual-run\.mjs/);
  assert.match(visual, /visual-diffs/);
  assert.match(visual, /timeout-minutes:\s*20/);
});

test("visual stub returns fixed items and rejects everything else", async () => {
  installVisualStub("populated");
  const page = await globalThis.__TAURI__.core.invoke("queue_query");
  const items = page.items;
  assert.equal(items.length, 2);
  assert.equal(items[0].id, "visual-item-1");
  assert.equal(items[1].sourceAppName, "TextEdit");
  assert.equal("createdAt" in items[0], false);
  await assert.rejects(
    globalThis.__TAURI__.core.invoke("preview_support_bundle"),
    /visual_stub/,
  );
  installVisualStub("empty");
  const none = await globalThis.__TAURI__.core.invoke("list_queue_items");
  assert.deepEqual(none, []);
});
