import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

test("quick panel is activating markup without a focus trap", () => {
  assert.doesNotMatch(html, /aria-modal\s*=\s*"true"/i);
  assert.doesNotMatch(html, /role\s*=\s*"dialog"/i);
  assert.match(html, /<main\b/);
  assert.match(html, /chrome\.css/);
  assert.match(html, /data-physical-edge="(left|right|top)"/);
  assert.doesNotMatch(
    html,
    /data-physical-edge="(start|end|leading|trailing)"/,
  );
  assert.equal(en["panel.quick.title"], "Bronze");
  assert.match(html, /data-i18n="panel.quick.title"/);
  assert.doesNotMatch(html, /data-open-window=/);
  assert.match(html, /data-i18n="panel.empty"/);
  assert.match(html, /data-i18n="copy.action.copy"/);
  assert.doesNotMatch(html.toLowerCase(), /copper|cooper/);
});

test("tauri conf shows the quick panel on launch", () => {
  const conf = JSON.parse(
    readFileSync(join(root, "../src-tauri/tauri.conf.json"), "utf8"),
  );
  const windows = conf.app.windows;
  const quick = windows.find((window) => window.label === "quick");
  const library = windows.find((window) => window.label === "library");
  const settings = windows.find((window) => window.label === "settings");
  assert.equal(quick.visible, true);
  assert.equal(quick.url, "index.html");
  assert.equal(quick.width, 400);
  assert.equal(quick.height, 720);
  assert.equal(quick.minWidth, 320);
  assert.equal(quick.minHeight, 560);
  assert.equal(library.visible, false);
  assert.equal(library.minWidth, 480);
  assert.equal(library.minHeight, 400);
  assert.equal(library.url, "library.html");
  assert.equal(settings.visible, false);
  assert.equal(settings.url, "settings.html");
  assert.equal(settings.minWidth, 480);
  assert.equal(settings.minHeight, 480);
  assert.equal(conf.app.withGlobalTauri, true);
});
