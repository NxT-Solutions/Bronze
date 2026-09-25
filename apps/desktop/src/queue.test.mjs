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

test("queue list uses semantic list article and real buttons", () => {
  assert.match(html, /<ul\b[^>]*id="queue"/);
  assert.match(html, /<article lang="und" dir="auto">/);
  assert.match(
    html,
    /<button\b[^>]*type="button"[^>]*data-i18n-aria-label="queue.item.moveUp"/,
  );
  assert.match(html, /data-i18n-aria-label="queue.item.moveDown"/);
  assert.match(html, /data-slot="action-icons"/);
  assert.doesNotMatch(html, /data-slot="toolbar-overflow"/);
  assert.doesNotMatch(html, /<div\b[^>]*(onclick|role="button")/);
  assert.equal(en["queue.item.moveUp"], "Move up");
  assert.equal(en["queue.item.moveDown"], "Move down");
  assert.doesNotMatch(html, /data-queue-action="moveUp"[^>]*\bdisabled\b/);
  assert.doesNotMatch(html, /data-queue-action="moveDown"[^>]*\bdisabled\b/);
  assert.equal(en["queue.item.complete"], "Complete");
  assert.match(html, /data-i18n="capture.source"/);
  assert.match(html, /<h3\b[^>]*data-slot="title"/);
  assert.match(html, /<button\b[^>]*type="button"[^>]*data-slot="expand"/);
  assert.equal(en["capture.source"], "From {appName}");
  assert.equal(en["queue.item.showMore"], "Show more");
  assert.equal(en["queue.list.loading"], "Loading more items.");
  assert.equal(en["queue.item.showLess"], "Show less");
  assert.equal(en["queue.item.title"], "Item");
});
