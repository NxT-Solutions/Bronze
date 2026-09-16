import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { composerShouldSubmit, QUE_007_COMPLETE } from "./queue-live.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const live = readFileSync(join(root, "queue-live.mjs"), "utf8");

test("composer submit is Cmd-Enter or the form and live queue is wired", () => {
  assert.equal(composerShouldSubmit({ type: "submit" }), true);
  assert.equal(
    composerShouldSubmit({ type: "keydown", key: "Enter", metaKey: true }),
    true,
  );
  assert.equal(
    composerShouldSubmit({ type: "keydown", key: "Enter", ctrlKey: true }),
    true,
  );
  assert.equal(
    composerShouldSubmit({ type: "keydown", key: "Enter", metaKey: false }),
    false,
  );
  assert.equal(QUE_007_COMPLETE, false);
  assert.match(html, /queue-live\.mjs/);
  assert.match(live, /list_overview_items/);
  assert.match(live, /queue-changed/);
  assert.match(html, /data-i18n="panel.empty"/);
  assert.doesNotMatch(html, /data-open-window=/);
});
