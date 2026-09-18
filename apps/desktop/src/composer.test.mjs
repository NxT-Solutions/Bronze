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

test("composer markup is a labeled textbox with catalog strings", () => {
  assert.match(html, /<form\b[^>]*id="composer"/);
  assert.match(html, /id="composer-label"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /id="composer-body"/);
  assert.match(html, /role="textbox"/);
  assert.match(html, /aria-multiline="true"/);
  assert.match(html, /aria-labelledby="composer-label"/);
  assert.match(html, /contenteditable="true"/);
  assert.match(html, /role="toolbar"/);
  assert.match(html, /data-composer-format="strong"/);
  assert.match(html, /data-composer-format="em"/);
  assert.match(html, /data-composer-format="ul"/);
  assert.match(html, /data-composer-format="ol"/);
  assert.match(html, /data-i18n-aria-label="composer.format.bold"/);
  assert.match(html, /data-i18n-title="composer.format.bold.tip"/);
  assert.doesNotMatch(html, /<kbd/);
  assert.match(html, />B<\/span>/);
  assert.match(html, />I<\/span>/);
  assert.match(html, /<button\b[^>]*type="submit"/);
  assert.match(html, /data-i18n="composer.add.label"/);
  assert.match(html, /data-i18n="composer.add.submit"/);
  assert.match(html, /data-i18n-placeholder="composer.placeholder"/);
  assert.match(html, /aria-placeholder="Type or Capture"/);
  assert.equal(en["composer.add.label"], "Add item");
  assert.equal(en["composer.placeholder"], "Type or Capture");
  assert.equal(en["composer.add.submit"], "Add");
  assert.equal(en["composer.add.submit.chord"], "Add ⇧↩");
  assert.equal(en["composer.add.error"], "Could not add.");
  assert.equal(en["composer.format.bold"], "Bold");
  assert.equal(en["composer.format.italic"], "Italic");
  assert.equal(en["composer.format.bold.tip"], "Bold ⌘B");
  assert.equal(en["composer.format.italic.tip"], "Italic ⌘I");
  assert.equal(en["panel.empty"], "Select text and Capture, or type here.");
});
