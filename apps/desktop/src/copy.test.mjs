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

test("copy toolbar uses a named profile and has no synthetic paste", () => {
  assert.match(html, /id="copy-toolbar"/);
  assert.match(html, /<select\b[^>]*id="output-profile"/);
  assert.match(html, /data-i18n="copy.action.copy"/);
  assert.doesNotMatch(html, /synthetic[- ]paste/i);
  assert.equal(en["copy.action.copy"], "Copy");
  assert.equal(en["copy.profile.label"], "Output profile");
});
