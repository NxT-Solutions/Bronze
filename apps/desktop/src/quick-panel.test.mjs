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
});
