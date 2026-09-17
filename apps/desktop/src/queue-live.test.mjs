import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyCaptureResult,
  captureFeedbackKey,
  composerShouldSubmit,
  formatCaptureSource,
  QUE_007_COMPLETE,
} from "./queue-live.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const live = readFileSync(join(root, "queue-live.mjs"), "utf8");
const itemView = readFileSync(join(root, "item-view.mjs"), "utf8");

const FEEDBACK = {
  "capture.announce.saved": "Captured to Bronze.",
  "capture.announce.rejected":
    "No selection was captured; select text in another app.",
  "capture.announce.denied": "Accessibility is required to read the selection.",
  "capture.announce.protected": "That field is protected and was not captured.",
  "capture.announce.failed": "Capture did not save.",
};

function captureStatusRoot(messages = FEEDBACK) {
  const status = { textContent: "", hidden: true };
  return {
    status,
    querySelector(sel) {
      if (sel === "#capture-status") {
        return status;
      }
      const key = sel.match(/data-i18n="([^"]+)"/)?.[1];
      if (key && Object.hasOwn(messages, key)) {
        return { textContent: messages[key] };
      }
      return null;
    },
  };
}

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
  assert.match(live, /capture-result/);
  assert.match(html, /id="capture-status"/);
  assert.match(html, /role="status"/);
  assert.match(html, /data-i18n="capture.announce.rejected"/);
  assert.match(html, /data-i18n="capture.announce.denied"/);
  assert.match(html, /data-i18n="capture.announce.protected"/);
  assert.match(html, /data-i18n="capture.announce.failed"/);
  assert.match(html, /data-i18n="copy.announce.copied"/);
  assert.match(html, /data-i18n="copy.announce.failed"/);
  assert.match(html, /id="action-status"/);
  assert.match(live, /runBusy/);
  assert.match(live, /applyActionStatus/);
  assert.match(chrome, /#capture-status\[hidden\]/);
  assert.match(chrome, /\[data-capture-message\]\[hidden\]/);
  assert.match(html, /data-i18n="panel.empty"/);
  assert.match(html, /data-i18n="capture.source"/);
  assert.match(html, /data-slot="source"/);
  assert.match(html, /data-slot="source-icon"/);
  assert.match(html, /data-slot="source-label"/);
  assert.match(html, /data-slot="title"/);
  assert.match(html, /data-slot="expand"/);
  assert.match(html, /data-i18n="queue.item.showMore"/);
  assert.match(html, /data-i18n="queue.item.showLess"/);
  assert.match(live, /sourceAppName/);
  assert.match(live, /sourceAppIcon/);
  assert.match(live, /applySourceRow/);
  assert.match(live, /fillItemChrome/);
  assert.doesNotMatch(live, /http:\/\//);
  assert.match(itemView, /renderMarkdownBody/);
  assert.match(itemView, /is-expanded/);
  assert.doesNotMatch(live, /data-slot=body"\]\.textContent = item\.body/);
  assert.doesNotMatch(live, /innerHTML = item\.body/);
  assert.doesNotMatch(itemView, /innerHTML = item\.body/);
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

test("capture feedback maps terminal reason to catalog keys and status text", () => {
  assert.equal(
    captureFeedbackKey({ terminal: "saved", reason: "ok" }),
    "capture.announce.saved",
  );
  assert.equal(
    captureFeedbackKey({ terminal: "rejected", reason: "no_selection" }),
    "capture.announce.rejected",
  );
  assert.equal(
    captureFeedbackKey({ terminal: "rejected", reason: "accessibility" }),
    "capture.announce.denied",
  );
  assert.equal(
    captureFeedbackKey({ terminal: "rejected", reason: "protected" }),
    "capture.announce.protected",
  );
  assert.equal(
    captureFeedbackKey({ terminal: "failed" }),
    "capture.announce.failed",
  );
  assert.equal(
    captureFeedbackKey({ terminal: "cancelled", reason: "failed" }),
    "capture.announce.failed",
  );
  const inbox = captureStatusRoot();
  applyCaptureResult(inbox, { terminal: "saved", reason: "ok" });
  assert.equal(inbox.status.textContent, FEEDBACK["capture.announce.saved"]);
  assert.equal(inbox.status.hidden, false);
  applyCaptureResult(inbox, {
    terminal: "rejected",
    reason: "no_selection",
  });
  assert.equal(inbox.status.textContent, FEEDBACK["capture.announce.rejected"]);
  assert.equal(inbox.status.hidden, false);
  applyCaptureResult(inbox, {
    terminal: "rejected",
    reason: "accessibility",
  });
  assert.equal(inbox.status.textContent, FEEDBACK["capture.announce.denied"]);
  applyCaptureResult(inbox, { terminal: "rejected", reason: "protected" });
  assert.equal(
    inbox.status.textContent,
    FEEDBACK["capture.announce.protected"],
  );
  applyCaptureResult(inbox, { terminal: "failed", reason: "failed" });
  assert.equal(inbox.status.textContent, FEEDBACK["capture.announce.failed"]);
  const empty = captureStatusRoot({});
  applyCaptureResult(empty, { terminal: "saved", reason: "ok" });
  assert.equal(empty.status.textContent, "");
  assert.equal(empty.status.hidden, true);
});
