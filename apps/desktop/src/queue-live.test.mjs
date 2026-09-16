import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  composerShouldSubmit,
  formatCaptureSource,
  QUE_007_COMPLETE,
} from "./queue-live.mjs";

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
  assert.match(html, /data-i18n="capture.source"/);
  assert.match(html, /data-slot="source"/);
  assert.match(live, /sourceAppName/);
  assert.doesNotMatch(html, /data-open-window=/);
});

test("capture source uses the catalog placeholder and stays unavailable without an app name", () => {
  assert.equal(
    formatCaptureSource("From {appName}", "TextEdit"),
    "From TextEdit",
  );
  assert.equal(
    formatCaptureSource("【FFrróomm {appName} [·]  [·] 】", "Claude"),
    "【FFrróomm Claude [·]  [·] 】",
  );
  assert.equal(formatCaptureSource(" morF⁧{appName}⁩", "Notes"), " morF⁧Notes⁩");
  assert.equal(formatCaptureSource("From {appName}", null), null);
  assert.equal(formatCaptureSource("From {appName}", undefined), null);
  assert.equal(formatCaptureSource("From {appName}", ""), null);
  assert.equal(formatCaptureSource("From {appName}", "   "), null);
  assert.equal(formatCaptureSource("From", "TextEdit"), null);
});
