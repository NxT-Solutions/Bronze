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
  assert.match(html, /padding-inline/);
  assert.match(html, /data-physical-edge="(left|right|top)"/);
  assert.doesNotMatch(
    html,
    /data-physical-edge="(start|end|leading|trailing)"/,
  );
  assert.equal(en["panel.quick.title"], "Bronze");
  assert.match(html, /data-i18n="panel.quick.title"/);
  assert.match(html, /data-open-window="library"/);
  assert.match(html, /data-open-window="settings"/);
  assert.match(html, /data-i18n="library.title"/);
  assert.match(html, /data-i18n="settings.title"/);
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
  assert.equal(library.visible, false);
  assert.equal(library.url, "library.html");
  assert.equal(settings.visible, false);
  assert.equal(settings.url, "settings.html");
  assert.equal(conf.app.withGlobalTauri, true);
});
