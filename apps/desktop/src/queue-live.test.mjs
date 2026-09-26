import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyCaptureResult,
  applyLiveWindow,
  applyQueuePage,
  captureFeedbackKey,
  composerFormatAction,
  composerHotkey,
  composerIsMultiline,
  composerShouldSubmit,
  composerSubmitLabelKey,
  formatCaptureSource,
  liveQueueFetchPlan,
  QUE_007_COMPLETE,
  QUEUE_PAGE_SIZE,
  queueFocusAtEnd,
  queueMoveDisabled,
  syncQueueMoveAvailability,
} from "./queue-live.mjs";
import { planNewItemFollow } from "./queue-reveal.mjs";
import { queueListArgs, queueSortFromEvent } from "./queue-sort.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const live = readFileSync(join(root, "queue-live.mjs"), "utf8");
const motion = readFileSync(join(root, "queue-motion.mjs"), "utf8");
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
  assert.match(live, /queue_query/);
  assert.doesNotMatch(live, /list_overview_items/);
  assert.equal(QUEUE_PAGE_SIZE, 20);
  assert.doesNotMatch(live, /OFFSET/);
  assert.match(live, /queueListArgs/);
  assert.match(live, /resetPage/);
  assert.match(live, /listenQueueSortChanged/);
  assert.match(live, /queue-changed/);
  const cursor = "v2.newest.20.65";
  assert.deepEqual(queueListArgs("newest", cursor), {
    sort: "newest",
    cursor,
  });
  assert.deepEqual(queueListArgs("oldest", cursor), {
    sort: "oldest",
    cursor: null,
  });
  assert.deepEqual(queueListArgs("newest", null), {
    sort: "newest",
    cursor: null,
  });
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
  assert.match(live, /bindIconTips/);
  assert.match(live, /syncQueueMoveAvailability/);
  assert.match(live, /observeQueueOrder/);
  assert.match(html, /data-slot="action-icons"/);
  assert.match(html, /data-queue-action="complete"/);
  assert.match(html, /data-queue-action="skip"/);
  assert.match(html, /data-queue-action="edit"/);
  assert.match(html, /data-queue-action="moveUp"/);
  assert.match(html, /data-queue-action="moveDown"/);
  assert.match(html, /data-queue-action="trash"/);
  assert.match(html, /data-queue-action="copy"/);
  assert.doesNotMatch(html, /data-slot="toolbar-overflow"/);
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
  assert.match(motion, /sourceAppName/);
  assert.match(motion, /sourceAppIcon/);
  assert.match(motion, /applySourceRow/);
  assert.match(motion, /fillItemChrome/);
  assert.match(
    motion,
    /function placeQueueNode\(list, node\) \{\n\s+list\.append\(node\);\n\s+syncExpandVisibility/,
  );
  assert.match(motion, /is-entering/);
  assert.match(live, /queue-motion\.mjs/);
  assert.match(live, /applyQueueItemMutation/);
  assert.match(live, /refresh\(\{ action/);
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

function fakeMoveButton(action, label) {
  return {
    dataset: { queueAction: action },
    disabled: false,
    getAttribute(name) {
      if (name === "aria-label") {
        return label;
      }
      return null;
    },
  };
}

function fakeQueueRow() {
  const moveUp = fakeMoveButton("moveUp", "Move up");
  const moveDown = fakeMoveButton("moveDown", "Move down");
  const copy = fakeMoveButton("copy", "Copy");
  return {
    moveUp,
    moveDown,
    copy,
    querySelector(sel) {
      if (sel === '[data-queue-action="moveUp"]') {
        return moveUp;
      }
      if (sel === '[data-queue-action="moveDown"]') {
        return moveDown;
      }
      if (sel === '[data-queue-action="copy"]') {
        return copy;
      }
      return null;
    },
  };
}

test("move up and down disable at the visible ends and stay named", () => {
  assert.deepEqual(queueMoveDisabled(0, 3), { moveUp: true, moveDown: false });
  assert.deepEqual(queueMoveDisabled(1, 3), { moveUp: false, moveDown: false });
  assert.deepEqual(queueMoveDisabled(2, 3), { moveUp: false, moveDown: true });
  assert.deepEqual(queueMoveDisabled(0, 1), { moveUp: true, moveDown: true });
  const first = fakeQueueRow();
  const middle = fakeQueueRow();
  const last = fakeQueueRow();
  const list = { children: [first, middle, last] };
  syncQueueMoveAvailability(list);
  assert.equal(first.moveUp.disabled, true);
  assert.equal(first.moveDown.disabled, false);
  assert.equal(middle.moveUp.disabled, false);
  assert.equal(middle.moveDown.disabled, false);
  assert.equal(last.moveUp.disabled, false);
  assert.equal(last.moveDown.disabled, true);
  assert.equal(first.copy.disabled, false);
  assert.equal(first.moveUp.getAttribute("aria-label"), "Move up");
  assert.equal(last.moveDown.getAttribute("aria-label"), "Move down");
  const only = fakeQueueRow();
  syncQueueMoveAvailability({ children: [only] });
  assert.equal(only.moveUp.disabled, true);
  assert.equal(only.moveDown.disabled, true);
  assert.equal(only.copy.disabled, false);
  list.children = [last, first, middle];
  syncQueueMoveAvailability(list);
  assert.equal(last.moveUp.disabled, true);
  assert.equal(last.moveDown.disabled, false);
  assert.equal(first.moveUp.disabled, false);
  assert.equal(first.moveDown.disabled, false);
  assert.equal(middle.moveUp.disabled, false);
  assert.equal(middle.moveDown.disabled, true);
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

test("queue pages keep order when a capture is prepended (QUE-002, QUE-008, A11Y-001, I18N-001)", () => {
  const page = {
    items: [
      { id: "b", body: "b" },
      { id: "c", body: "c" },
    ],
    nextCursor: "cursor-after-c",
  };
  let state = applyQueuePage(
    { items: [], nextCursor: null, anchorCursor: null },
    page,
    "head",
  );
  state = applyQueuePage(
    state,
    {
      items: [
        { id: "a", body: "a" },
        { id: "b", body: "b2" },
      ],
      nextCursor: "shifted",
    },
    "head",
  );
  assert.deepEqual(
    state.items.map((item) => item.id),
    ["a", "b", "c"],
  );
  assert.equal(state.items[1].body, "b2");
  assert.equal(state.nextCursor, "cursor-after-c");
  state = applyQueuePage(
    state,
    {
      items: [
        { id: "c", body: "c" },
        { id: "d", body: "d" },
      ],
      nextCursor: null,
    },
    "more",
  );
  assert.deepEqual(
    state.items.map((item) => item.id),
    ["a", "b", "c", "d"],
  );
  assert.equal(new Set(state.items.map((item) => item.id)).size, 4);
  const last = {
    contains(node) {
      return node === "inside";
    },
  };
  assert.equal(queueFocusAtEnd([{}, last], "inside"), true);
  assert.equal(queueFocusAtEnd([{}, last], "elsewhere"), false);
  assert.match(html, /data-i18n="queue.list.loading"/);
  assert.match(html, /Loading more items\./);
  assert.match(html, /id="queue-more-status"[^>]*role="status"/);
  assert.match(html, /data-queue-sentinel/);
  assert.match(html, /data-queue-action="copy"/);
  assert.match(html, /data-i18n="queue.item.showMore"/);
  assert.match(html, /data-slot="source"/);
  assert.doesNotMatch(html, /data-queue-sentinel[^>]*onclick/);
  assert.doesNotMatch(
    chrome,
    /\[data-queue-sentinel\][^{]*\{[^}]*outline:\s*none/,
  );
  assert.match(live, /focusin/);
  assert.match(live, /IntersectionObserver/);
  assert.match(live, /rootMargin: "240px"/);
});

test("a live insert updates the loaded edge and leaves a later page alone", () => {
  const newestLoaded = {
    items: [
      { id: "b", status: "queued", title: "B" },
      { id: "c", status: "queued", title: "C" },
    ],
    nextCursor: "v2.newest.2.63",
    pagesLoaded: 1,
  };
  assert.deepEqual(liveQueueFetchPlan(newestLoaded), {
    pages: 1,
    extendEnd: false,
  });
  const newest = applyLiveWindow([
    {
      items: [
        { id: "a", status: "queued", title: "A" },
        { id: "b", status: "queued", title: "B2" },
      ],
      nextCursor: "v2.newest.3.62",
    },
  ]);
  assert.deepEqual(
    newest.items.map((item) => item.id),
    ["a", "b"],
  );
  assert.equal(newest.items[1].title, "B2");
  assert.equal(
    planNewItemFollow({
      sort: "newest",
      nearStart: true,
      prevIds: ["b", "c"],
      nextIds: newest.items.map((item) => item.id),
    }).scrollId,
    "a",
  );
  assert.equal(
    planNewItemFollow({
      sort: "newest",
      nearStart: false,
      prevIds: ["b", "c"],
      nextIds: newest.items.map((item) => item.id),
    }).scrollId,
    null,
  );

  const oldestOpen = {
    items: [
      { id: "a", status: "queued" },
      { id: "b", status: "queued" },
    ],
    nextCursor: "v2.oldest.2.62",
    pagesLoaded: 1,
  };
  assert.equal(liveQueueFetchPlan(oldestOpen).extendEnd, false);
  const untouched = applyLiveWindow([
    {
      items: [
        { id: "a", status: "queued" },
        { id: "b", status: "queued" },
      ],
      nextCursor: "v2.oldest.2.62",
    },
  ]);
  assert.deepEqual(
    untouched.items.map((item) => item.id),
    ["a", "b"],
  );
  assert.equal(untouched.nextCursor, "v2.oldest.2.62");

  const oldestEnd = {
    items: [
      { id: "a", status: "queued" },
      { id: "b", status: "queued" },
    ],
    nextCursor: null,
    pagesLoaded: 1,
  };
  assert.deepEqual(liveQueueFetchPlan(oldestEnd), {
    pages: 1,
    extendEnd: true,
  });
  const appended = applyLiveWindow([
    {
      items: [
        { id: "a", status: "queued" },
        { id: "b", status: "queued" },
      ],
      nextCursor: "v2.oldest.2.62",
    },
    {
      items: [{ id: "n", status: "queued", title: "New" }],
      nextCursor: null,
    },
  ]);
  assert.deepEqual(
    appended.items.map((item) => item.id),
    ["a", "b", "n"],
  );
  assert.equal(appended.pagesLoaded, 2);
  assert.equal(
    planNewItemFollow({
      sort: "oldest",
      nearStart: true,
      prevIds: ["a", "b"],
      nextIds: appended.items.map((item) => item.id),
    }).scrollId,
    null,
  );
  assert.match(live, /liveQueueFetchPlan/);
  assert.match(live, /extendEnd/);
  assert.match(live, /applyLiveWindow/);
  assert.match(live, /fetchLiveWindow/);
});

test("a status change patches the visible row without moving it", () => {
  const patched = applyLiveWindow([
    {
      items: [{ id: "a", status: "queued", title: "Older" }],
      nextCursor: "v2.oldest.1.61",
    },
    {
      items: [{ id: "b", status: "copied", title: "Renamed" }],
      nextCursor: null,
    },
  ]);
  assert.equal(patched.items[1].id, "b");
  assert.equal(patched.items[1].status, "copied");
  assert.equal(patched.items[1].title, "Renamed");
  assert.deepEqual(
    patched.items.map((item) => item.id),
    ["a", "b"],
  );
  assert.equal(queueSortFromEvent("oldest"), "oldest");
  assert.equal(queueSortFromEvent({ sort: "newest" }), "newest");
  assert.equal(queueSortFromEvent(null), null);
  assert.match(live, /queueSortFromEvent/);
  const copyAt = live.indexOf('action === "copy"');
  const assignAt = live.indexOf("lastCopiedId = id", copyAt);
  const invokeAt = live.indexOf("copy_queue_items", copyAt);
  assert.ok(copyAt >= 0 && assignAt > copyAt && assignAt < invokeAt);
});
