import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyCaptureResult,
  captureFeedbackKey,
  composerFormatAction,
  composerHotkey,
  composerIsMultiline,
  composerShouldSubmit,
  composerSubmitLabelKey,
  formatCaptureSource,
  QUE_007_COMPLETE,
} from "./queue-live.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const live = readFileSync(join(root, "queue-live.mjs"), "utf8");
const itemView = readFileSync(join(root, "item-view.mjs"), "utf8");

const FEEDBACK = {
  "capture.announce.saved": "Saved.",
  "capture.announce.rejected": "No text selected.",
  "capture.announce.denied": "Allow Accessibility to capture.",
  "capture.announce.protected": "Protected field.",
  "capture.announce.failed": "Could not save.",
  "capture.announce.excluded": "This app is excluded.",
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

test("composer submit is Shift-Enter or the form and live queue is wired", () => {
  assert.equal(composerShouldSubmit({ type: "submit" }), true);
  assert.equal(
    composerShouldSubmit({ type: "keydown", key: "Enter", shiftKey: true }),
    true,
  );
  assert.equal(
    composerShouldSubmit({ type: "keydown", key: "Enter", metaKey: true }),
    false,
  );
  assert.equal(
    composerShouldSubmit({ type: "keydown", key: "Enter", ctrlKey: true }),
    false,
  );
  assert.equal(
    composerShouldSubmit({ type: "keydown", key: "Enter", shiftKey: false }),
    false,
  );
  assert.equal(composerIsMultiline("one"), false);
  assert.equal(composerIsMultiline("one\ntwo"), true);
  assert.equal(composerSubmitLabelKey("one"), "composer.add.submit");
  assert.equal(composerSubmitLabelKey("one\ntwo"), "composer.add.submit.chord");
  assert.equal(QUE_007_COMPLETE, false);
  assert.match(html, /queue-live\.mjs/);
  assert.match(live, /list_overview_items/);
  assert.match(live, /queue-changed/);
  assert.match(live, /capture-result/);
  assert.match(live, /LOCALE_APPLIED_EVENT/);
  assert.match(html, /id="capture-status"[^>]*visually-hidden/);
  assert.match(html, /id="chrome-notice"/);
  assert.match(html, /data-notice-dismiss/);
  assert.match(html, /chrome.notice.dismiss/);
  assert.match(html, /role="status"/);
  assert.match(html, /data-i18n="capture.announce.rejected"/);
  assert.match(html, /data-i18n="capture.announce.denied"/);
  assert.match(html, /data-i18n="capture.announce.protected"/);
  assert.match(html, /data-i18n="capture.announce.failed"/);
  assert.match(html, /data-i18n="capture.announce.excluded"/);
  assert.match(html, /data-i18n="copy.announce.copied"/);
  assert.match(html, /data-i18n="copy.announce.failed"/);
  assert.match(html, /id="action-status"/);
  assert.match(live, /runBusy/);
  assert.match(live, /applyActionStatus/);
  assert.match(live, /bindChromeNotice/);
  assert.match(live, /showChromeNotice/);
  assert.match(live, /bindOverflowDismiss/);
  assert.match(live, /copy\.announce\.copied", button/);
  assert.match(html, /data-slot="action-tip"/);
  assert.match(html, /id="action-status"[^>]*visually-hidden/);
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
  assert.match(live, /is-entering/);
  assert.doesNotMatch(live, /http:\/\//);
  assert.match(itemView, /renderMarkdownBody/);
  assert.match(itemView, /is-expanded/);
  assert.doesNotMatch(live, /data-slot=body"\]\.textContent = item\.body/);
  assert.doesNotMatch(live, /innerHTML = item\.body/);
  assert.doesNotMatch(itemView, /innerHTML = item\.body/);
  assert.doesNotMatch(html, /data-open-window=/);
  assert.doesNotMatch(live, /\.prompt/);
  assert.match(live, /serializeComposerDom/);
  assert.match(live, /applyComposerFormat/);
  assert.match(live, /applyComposerList/);
  assert.match(live, /composerHotkey/);
  assert.match(html, /contenteditable="true"/);
  assert.match(html, /data-composer-format="strong"/);
  assert.match(live, /openEditSheet/);
  assert.match(html, /id="edit-sheet"/);
  assert.match(html, /data-edit-dismiss/);
  assert.match(html, /data-i18n="queue.item.edit.save"/);
});

test("composer hotkeys follow common rich-text chords", () => {
  assert.equal(
    composerHotkey({ key: "b", metaKey: true, isComposing: false }),
    "strong",
  );
  assert.equal(
    composerHotkey({ key: "i", ctrlKey: true, isComposing: false }),
    "em",
  );
  assert.equal(
    composerHotkey({
      key: "8",
      code: "Digit8",
      metaKey: true,
      shiftKey: true,
      isComposing: false,
    }),
    "ul",
  );
  assert.equal(
    composerHotkey({
      key: "7",
      code: "Digit7",
      metaKey: true,
      shiftKey: true,
      isComposing: false,
    }),
    "ol",
  );
  assert.equal(
    composerHotkey({ type: "keydown", key: "Enter", shiftKey: true }),
    "submit",
  );
  assert.equal(
    composerHotkey({ type: "keydown", key: "Enter", metaKey: true }),
    null,
  );
});

test("composer format stays on for the next typed characters", () => {
  assert.equal(
    composerFormatAction({ collapsed: true, alreadyOn: false }),
    "insert",
  );
  assert.equal(
    composerFormatAction({ collapsed: true, alreadyOn: true }),
    "exit",
  );
  assert.equal(
    composerFormatAction({ collapsed: false, alreadyOn: false }),
    "wrap",
  );
  assert.equal(
    composerFormatAction({ collapsed: false, alreadyOn: true }),
    "unwrap",
  );
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
    captureFeedbackKey({ terminal: "rejected", reason: "app_excluded" }),
    "capture.announce.excluded",
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
  applyCaptureResult(inbox, {
    terminal: "rejected",
    reason: "app_excluded",
  });
  assert.equal(inbox.status.textContent, FEEDBACK["capture.announce.excluded"]);
  applyCaptureResult(inbox, { terminal: "failed", reason: "failed" });
  assert.equal(inbox.status.textContent, FEEDBACK["capture.announce.failed"]);
  const empty = captureStatusRoot({});
  applyCaptureResult(empty, { terminal: "saved", reason: "ok" });
  assert.equal(empty.status.textContent, "");
  assert.equal(empty.status.hidden, true);
});
