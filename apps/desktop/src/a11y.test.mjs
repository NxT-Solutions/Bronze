import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const settings = readFileSync(join(root, "settings.html"), "utf8");
const library = readFileSync(join(root, "library.html"), "utf8");
const help = readFileSync(join(root, "help.html"), "utf8");

test("queue action accessible names contain visible labels", () => {
  assert.match(html, /data-i18n-aria-label="queue.item.moveUp"/);
  assert.match(html, /aria-label="Move up"/);
  assert.match(html, /data-i18n="queue.item.moveUp">Move up</);
  assert.match(html, /data-i18n-aria-label="queue.item.moveDown"/);
  assert.match(html, /aria-label="Move down"/);
  assert.match(html, /data-i18n="queue.item.moveDown">Move down</);
  assert.match(html, /data-i18n-aria-label="queue.item.complete"/);
  assert.match(html, /aria-label="Complete"/);
  assert.match(html, /data-i18n="queue.item.complete">Complete</);
  assert.match(html, /data-i18n-aria-label="queue.item.skip"/);
  assert.match(html, /aria-label="Skip"/);
  assert.match(html, /data-i18n="queue.item.skip">Skip</);
  assert.match(html, /data-i18n-aria-label="queue.item.trash"/);
  assert.match(html, /aria-label="Trash"/);
  assert.match(html, /data-i18n="queue.item.trash">Trash</);
  assert.match(html, /data-i18n-aria-label="queue.item.edit"/);
  assert.match(html, /aria-label="Edit…"/);
  assert.match(html, /data-i18n="queue.item.edit">Edit…</);
  assert.match(html, /data-i18n="queue.item.showMore">\s*Show more\s*</);
  assert.match(html, /data-i18n="queue.item.showLess">\s*Show less\s*</);
  assert.match(html, /data-i18n="queue.item.title">\s*Item\s*</);
});

test("four surfaces expose a skip link and section headings", () => {
  for (const page of [html, settings, library, help]) {
    assert.match(page, /class="skip-link"/);
    assert.match(
      page,
      /data-i18n="chrome.skip.toContent">\s*Skip to content\s*</,
    );
  }
  assert.match(html, /href="#composer-body"/);
  assert.match(html, /data-i18n="panel.section.active">\s*Inbox\s*</);
  assert.match(settings, /href="#settings-search"/);
  assert.match(library, /href="#library-nav"/);
  assert.match(library, /aria-labelledby="library-title"/);
  assert.match(library, /data-i18n="library.heading.items">\s*Items\s*</);
  assert.match(help, /href="#help-about"/);
  assert.match(help, /id="help-about"/);
});
