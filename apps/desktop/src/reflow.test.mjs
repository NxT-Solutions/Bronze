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
const xa = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en-XA/app.json"),
    "utf8",
  ),
);

test("320 CSS px reflow keeps one-axis scroll labeled overflow and composer", () => {
  assert.match(html, /overflow-x:\s*hidden/);
  assert.match(html, /flex-wrap:\s*wrap/);
  assert.match(html, /data-i18n="panel.toolbar.overflow"/);
  assert.match(html, /id="composer"/);
  assert.equal(en["panel.toolbar.overflow"], "More actions");
  assert.ok(
    xa["panel.toolbar.overflow"].length > en["panel.toolbar.overflow"].length,
  );
});
