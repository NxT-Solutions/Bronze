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
  assert.match(html, /data-i18n="queue.item.moveUp">\s*Move up\s*</);
  assert.match(html, /data-i18n="queue.item.moveDown">\s*Move down\s*</);
  assert.match(html, /data-i18n="queue.item.complete">\s*Complete\s*</);
  assert.match(html, /data-i18n="queue.item.skip">\s*Skip\s*</);
  assert.match(html, /data-i18n="queue.item.trash">\s*Trash\s*</);
  assert.match(html, /data-i18n="queue.item.edit">\s*Edit…\s*</);
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
