import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const localesRoot = join(root, "../../../packages/i18n/locales");
const locales = ["en", "nl", "fr", "de", "es", "it", "en-XA", "ar-XB"];
const nativeKeys = [
  "app.name",
  "menu.status.capture",
  "menu.status.newNote",
  "menu.status.show",
  "menu.status.settings",
  "menu.status.quit",
  "panel.quick.title",
];

test("shipped locales cover native menu InfoPlist and WebView keys", () => {
  for (const locale of locales) {
    const catalog = JSON.parse(
      readFileSync(join(localesRoot, locale, "app.json"), "utf8"),
    );
    for (const key of nativeKeys) {
      assert.ok(catalog[key], `${locale} missing ${key}`);
    }
    assert.ok(catalog["composer.add.label"]);
    assert.ok(catalog["composer.format.bold"]);
    assert.ok(catalog["composer.format.italic"]);
    assert.ok(catalog["capture.source"]);
    assert.ok(catalog["queue.item.showMore"]);
    assert.ok(catalog["queue.item.showLess"]);
    assert.ok(catalog["queue.item.title"]);
    assert.ok(catalog["settings.title"]);
    assert.ok(catalog["settings.field.locale"]);
    assert.equal(catalog["app.name"], catalog["app.name"]);
  }
  assert.match(html, /data-i18n="panel.quick.title"/);
  assert.match(html, /data-i18n="capture.source"/);
  assert.match(html, /data-physical-edge="left"/);
  assert.match(html, /<article lang="und" dir="auto">/);
  assert.match(html, /id="quick-panel"[^>]*dir="ltr"/);
  assert.match(
    readFileSync(join(root, "settings.html"), "utf8"),
    /<html lang="en">/,
  );
  const en = JSON.parse(
    readFileSync(join(localesRoot, "en", "app.json"), "utf8"),
  );
  assert.equal(en["capture.source"], "From {appName}");
  assert.match(en["settings.permission.inputMonitoring.why"], /\s/);
  assert.equal(
    en["settings.permission.inputMonitoring.why"],
    "Needed for the global capture chord",
  );
  assert.doesNotMatch(
    en["settings.permission.inputMonitoring.why"],
    /Needededfor/,
  );
});
