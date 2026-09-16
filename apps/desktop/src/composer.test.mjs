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

test("composer markup is a labeled textarea with catalog strings", () => {
  assert.match(html, /<form\b[^>]*id="composer"/);
  assert.match(html, /<label\b[^>]*for="composer-body"/);
  assert.match(html, /<textarea\b[^>]*id="composer-body"/);
  assert.match(html, /<button\b[^>]*type="submit"/);
  assert.match(html, /data-i18n="composer.add.label"/);
  assert.match(html, /data-i18n="composer.add.submit"/);
  assert.equal(en["composer.add.label"], "Add item");
  assert.equal(en["composer.add.submit"], "Add");
  assert.equal(en["composer.add.error"], "Could not add item");
});
