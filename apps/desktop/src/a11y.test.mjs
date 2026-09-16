import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");

test("queue action accessible names contain visible labels", () => {
  assert.match(html, /data-i18n="queue.item.moveUp">Move up</);
  assert.match(html, /data-i18n="queue.item.moveDown">\s*Move down/);
  assert.match(html, /data-i18n="queue.item.complete">\s*Complete/);
  assert.match(html, /data-i18n="queue.item.skip">Skip</);
  assert.match(html, /data-i18n="queue.item.trash">Trash</);
  assert.match(html, /data-i18n="queue.item.edit">Edit</);
});
