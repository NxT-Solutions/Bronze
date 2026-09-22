import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
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

test("320 CSS px reflow keeps one-axis scroll, wrapping icon row, and composer", () => {
  assert.match(chrome, /overflow-x:\s*hidden/);
  assert.match(chrome, /flex-wrap:\s*wrap/);
  assert.match(chrome, /#quick-panel[\s\S]*?flex-wrap:\s*nowrap/);
  assert.match(chrome, /\.row-action-icons[\s\S]*flex-wrap:\s*wrap/);
  assert.match(chrome, /word-break:\s*normal/);
  assert.match(chrome, /white-space:\s*nowrap/);
  assert.doesNotMatch(chrome, /word-break:\s*break-all/);
  assert.doesNotMatch(chrome, /overflow-wrap:\s*anywhere/);
  assert.match(html, /chrome\.css/);
  assert.match(html, /data-slot="action-icons"/);
  assert.doesNotMatch(html, /data-i18n="panel.toolbar.overflow"/);
  assert.match(html, /id="composer"/);
  assert.equal(en["panel.toolbar.overflow"], "More actions");
  assert.ok(
    xa["panel.toolbar.overflow"].length > en["panel.toolbar.overflow"].length,
  );
});
