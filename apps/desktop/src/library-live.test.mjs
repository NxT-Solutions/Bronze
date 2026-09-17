import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  ADR_018_STATUS,
  announceCount,
  QUE_007_COMPLETE,
} from "./library-live.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const live = readFileSync(join(root, "library-live.mjs"), "utf8");
const html = readFileSync(join(root, "library.html"), "utf8");

test("library search stays a substring placeholder", () => {
  assert.equal(announceCount(0), "0 items");
  assert.equal(announceCount(2), "2 items");
  assert.equal(QUE_007_COMPLETE, false);
  assert.equal(ADR_018_STATUS, "Proposed");
});

test("library items sanitize markdown and keep a title plus expand reader", () => {
  assert.match(live, /fillItemChrome/);
  assert.doesNotMatch(live, /body\.textContent = item\.body/);
  assert.doesNotMatch(live, /innerHTML = item\.body/);
  assert.match(html, /id="library-item-template"/);
  assert.match(html, /data-slot="title"/);
  assert.match(html, /data-slot="expand"/);
  assert.match(html, /data-i18n="queue.item.showMore"/);
  assert.match(html, /data-i18n="queue.item.showLess"/);
  assert.match(html, /data-slot="source"/);
  assert.match(html, /data-slot="source-icon"/);
  assert.match(live, /applySourceRow/);
  assert.match(live, /sourceAppIcon/);
  assert.match(live, /runBusy/);
  assert.match(live, /is-entering/);
});
