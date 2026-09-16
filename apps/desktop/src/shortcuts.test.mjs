import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "settings.html"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

const actions = [
  "app.togglePanel",
  "capture.selection",
  "capture.newNote",
  "queue.copy",
  "queue.copyWithProfile",
  "queue.copyAndAdvance",
  "queue.complete",
  "queue.edit",
  "queue.moveUp",
  "queue.moveDown",
  "queue.search",
  "queue.undo",
  "window.settings",
];

test("shortcut recorder lists every action and keeps capture.selection as standardChord", () => {
  assert.match(html, /data-slot="shortcut-recorder"/);
  assert.match(html, /data-standard-chord="capture.selection"/);
  for (const action of actions) {
    assert.match(html, new RegExp(`data-action="${action}"`));
  }
  assert.match(html, /data-i18n="settings.shortcuts.skipTest"/);
  assert.equal(en["settings.shortcuts.record"], "Record shortcut");
});
