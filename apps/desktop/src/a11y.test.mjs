import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");

test("queue action accessible names contain visible labels", () => {
  assert.match(html, /data-i18n="queue.item.moveUp">\s*Move up\s*</);
  assert.match(html, /data-i18n="queue.item.moveDown">\s*Move down\s*</);
  assert.match(html, /data-i18n="queue.item.complete">\s*Complete\s*</);
  assert.match(html, /data-i18n="queue.item.skip">\s*Skip\s*</);
  assert.match(html, /data-i18n="queue.item.trash">\s*Trash\s*</);
  assert.match(html, /data-i18n="queue.item.edit">\s*Edit\s*</);
  assert.match(html, /data-i18n="queue.item.showMore">\s*Show more\s*</);
  assert.match(html, /data-i18n="queue.item.showLess">\s*Show less\s*</);
  assert.match(html, /data-i18n="queue.item.title">\s*Item\s*</);
});
