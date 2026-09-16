import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "settings.html"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

const actions = [
  ["app.togglePanel", "Toggle panel"],
  ["capture.selection", "Capture selection"],
  ["capture.newNote", "New note"],
  ["queue.copy", "Copy"],
  ["queue.copyWithProfile", "Copy with profile"],
  ["queue.copyAndAdvance", "Copy and advance"],
  ["queue.complete", "Complete"],
  ["queue.edit", "Edit"],
  ["queue.moveUp", "Move up"],
  ["queue.moveDown", "Move down"],
  ["queue.search", "Search"],
  ["queue.undo", "Undo"],
  ["window.settings", "Open settings"],
];

test("shortcut recorder lists every action and keeps capture.selection as standardChord", () => {
  assert.match(html, /data-slot="shortcut-recorder"/);
  assert.match(html, /data-standard-chord="capture.selection"/);
  assert.match(html, /shortcut-record-row/);
  assert.match(
    chrome,
    /\.shortcut-record-row\s*\{[\s\S]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s+max-content/,
  );
  assert.equal([...html.matchAll(/Capture selection/g)].length, 1);
  assert.match(html, /data-i18n="settings.shortcuts.skipTest"/);
  assert.equal(en["settings.shortcuts.record"], "Record shortcut");
  assert.equal(en["settings.shortcuts.unassigned"], "Not assigned");
  assert.match(html, /data-i18n="settings.shortcuts.unassigned"/);
  assert.match(html, />\s*Not assigned\s*</);
  for (const [action, title] of actions) {
    const key = `settings.shortcuts.action.${action}`;
    assert.equal(en[key], title, key);
    assert.match(html, new RegExp(`data-action="${action}"`));
    assert.match(html, new RegExp(`data-i18n="${key.replace(/\./g, "\\.")}"`));
    assert.match(html, new RegExp(`>\\s*${title}\\s*<`));
    assert.doesNotMatch(
      html,
      new RegExp(`<li data-action="${action}">\\s*${action}\\s*</li>`),
    );
  }
  assert.doesNotMatch(html, /Needededfor|Shortcutts|Skipptestt/);
});
