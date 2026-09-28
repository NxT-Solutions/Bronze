import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { LOCALE_APPLIED_EVENT } from "./apply-locale.mjs";
import {
  applyCaptureResult,
  applyLiveWindow,
  applyQueuePage,
  bindQueueLive,
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
import {
  applyQueueSortControl,
  patchSettingsQueueSort,
  persistQueueSort,
  queueListArgs,
  queueSortFromEvent,
  queueSortStateKey,
} from "./queue-sort.mjs";
import { applySettingsForm } from "./settings-live.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const live = readFileSync(join(root, "queue-live.mjs"), "utf8");
const sortSrc = readFileSync(join(root, "queue-sort.mjs"), "utf8");
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
  assert.match(live, /requestSortReload/);
  assert.match(live, /swapSortPage/);
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
  assert.match(live, /loadFirstPage/);
  assert.match(live, /firstPageReady/);
  assert.match(live, /firstPageExpandPlan/);
  assert.match(live, /paintFirstPageExpand/);
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
  assert.match(html, /class="item-expand"/);
  assert.doesNotMatch(html, /class="btn-ghost"[^>]*data-slot="expand"/);
  assert.match(html, /data-i18n="queue.item.showMore"/);
  assert.match(html, /data-i18n="queue.item.showLess"/);
  assert.match(itemView, /sourceAppName/);
  assert.match(itemView, /sourceAppIcon/);
  assert.match(itemView, /applyItemSource/);
  assert.match(itemView, /applySourceRow/);
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
    "a",
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
  assert.equal(
    planNewItemFollow({
      sort: "oldest",
      prevIds: ["a", "b"],
      nextIds: appended.items.map((item) => item.id),
    }).room,
    "end",
  );
  assert.equal(
    planNewItemFollow({
      sort: "oldest",
      prevIds: ["a", "b"],
      nextIds: appended.items.map((item) => item.id),
      hasMore: true,
    }).room,
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
  assert.match(live, /persistQueueSort/);
  assert.match(sortSrc, /set_queue_sort/);
  assert.match(live, /#queue-sort-toggle/);
  const copyAt = live.indexOf('action === "copy"');
  const copyBlock = live.slice(
    copyAt,
    live.indexOf('if (action === "edit"', copyAt),
  );
  const invokeAt = live.indexOf("copy_queue_items", copyAt);
  assert.ok(copyAt >= 0 && invokeAt > copyAt);
  assert.match(copyBlock, /copy\.announce\.copied/);
  assert.doesNotMatch(copyBlock, /lastCopiedId = id/);
  assert.doesNotMatch(copyBlock, /markLastCopied/);
  assert.doesNotMatch(copyBlock, /pulseCopy/);
});

test("queue sort toggle writes newest then oldest and reloads the list", async () => {
  const en = JSON.parse(
    readFileSync(
      join(root, "../../../packages/i18n/locales/en/app.json"),
      "utf8",
    ),
  );
  assert.equal(en["queue.sort.showOldest"], "Show oldest first");
  assert.equal(en["queue.sort.showNewest"], "Show newest first");
  assert.match(html, /id="queue-sort-toggle"/);
  assert.match(html, /<button\b[^>]*type="button"[^>]*id="queue-sort-toggle"/);
  assert.match(html, /data-i18n-aria-label="queue.sort.showOldest"/);
  assert.match(html, /aria-label="Show oldest first"/);
  assert.match(html, /aria-describedby="queue-sort-state"/);
  assert.match(
    html,
    /id="queue-sort-state"[\s\S]*data-i18n="settings.field.queueSort.newest"[\s\S]*>Newest first</,
  );
  assert.match(html, /data-slot="sort-tip-visual"/);
  assert.doesNotMatch(html, /id="queue-sort-toggle"[\s\S]{0,240}title=/);
  const composerEnd = html.indexOf("</form>");
  const sortBar = html.indexOf("queue-sort-bar");
  const queueId = html.indexOf('id="queue"');
  assert.ok(composerEnd > 0 && sortBar > composerEnd && sortBar < queueId);
  assert.match(chrome, /\.queue-sort-bar[\s\S]*justify-content:\s*flex-end/);
  assert.match(chrome, /\.queue-sort-toggle[\s\S]*--control-h/);
  assert.match(chrome, /\.queue-sort-toggle[\s\S]*--radius-control/);
  assert.match(chrome, /--sort-flip:\s*200ms/);
  assert.match(
    chrome,
    /html\[data-motion="reduce"\] \.queue-sort-toggle \[data-sort-glyph\][\s\S]*transition:\s*none/,
  );
  assert.doesNotMatch(
    chrome,
    /\.queue-sort-toggle[\s\S]{0,240}outline:\s*none/,
  );

  const tip = { id: "queue-sort-state", textContent: "", _attr: {} };
  tip.setAttribute = (name, value) => {
    tip._attr[name] = String(value);
  };
  const visual = { textContent: "", _attr: {} };
  visual.setAttribute = (name, value) => {
    visual._attr[name] = String(value);
  };
  const button = {
    dataset: {},
    _attr: {},
    getAttribute(name) {
      return Object.hasOwn(this._attr, name) ? this._attr[name] : null;
    },
    setAttribute(name, value) {
      this._attr[name] = String(value);
    },
    removeAttribute(name) {
      delete this._attr[name];
    },
    querySelector(sel) {
      if (sel === "[data-slot=sort-tip]") {
        return tip;
      }
      if (sel === "[data-slot=sort-tip-visual]") {
        return visual;
      }
      return null;
    },
  };
  assert.equal(queueSortStateKey("newest"), "settings.field.queueSort.newest");
  assert.equal(queueSortStateKey("oldest"), "settings.field.queueSort.oldest");
  applyQueueSortControl(button, "newest", en);
  assert.equal(button.dataset.queueSort, "newest");
  assert.equal(button.getAttribute("aria-label"), en["queue.sort.showOldest"]);
  assert.equal(
    button.getAttribute("data-i18n-aria-label"),
    "queue.sort.showOldest",
  );
  assert.equal(button.getAttribute("aria-describedby"), "queue-sort-state");
  assert.equal(tip.textContent, en["settings.field.queueSort.newest"]);
  assert.equal(visual.textContent, en["settings.field.queueSort.newest"]);
  assert.equal(button.getAttribute("title"), null);
  applyQueueSortControl(button, "oldest", en);
  assert.equal(button.getAttribute("aria-label"), en["queue.sort.showNewest"]);
  assert.equal(tip.textContent, en["settings.field.queueSort.oldest"]);
  assert.equal(visual.textContent, en["settings.field.queueSort.oldest"]);

  let store = {
    general: {},
    copy: { defaultProfileId: "plain", queueSort: "oldest" },
    data: {},
    privacy: {},
  };
  const saved = [];
  const queried = [];
  const invoke = async (cmd, args) => {
    if (cmd === "load_settings_v1") {
      return structuredClone(store);
    }
    if (cmd === "set_queue_sort") {
      store = patchSettingsQueueSort(store, args.sort);
      saved.push(store.copy.queueSort);
      return store;
    }
    if (cmd === "queue_query") {
      queried.push(args.sort);
      return { items: [{ id: args.sort, body: args.sort }], nextCursor: null };
    }
    if (cmd === "take_notice_activation") {
      return null;
    }
    throw new Error(cmd);
  };

  let settings = await invoke("load_settings_v1");
  settings = await persistQueueSort(invoke, settings, "newest");
  assert.equal(settings.copy.queueSort, "newest");
  settings = await persistQueueSort(invoke, settings, "oldest");
  assert.equal(settings.copy.queueSort, "oldest");
  assert.deepEqual(saved, ["newest", "oldest"]);
  const newestPage = await invoke("queue_query", {
    filter: "overview",
    limit: QUEUE_PAGE_SIZE,
    ...queueListArgs("newest", null),
  });
  const oldestPage = await invoke("queue_query", {
    filter: "overview",
    limit: QUEUE_PAGE_SIZE,
    ...queueListArgs("oldest", null),
  });
  assert.equal(newestPage.items[0].id, "newest");
  assert.equal(oldestPage.items[0].id, "oldest");
  assert.deepEqual(queried, ["newest", "oldest"]);

  const queueSort = { value: "newest" };
  applySettingsForm(
    {
      querySelector(sel) {
        return sel === "#queue-sort" ? queueSort : null;
      },
    },
    settings,
  );
  assert.equal(queueSort.value, "oldest");
});

function createMountDocument() {
  if (typeof globalThis.HTMLButtonElement !== "function") {
    globalThis.HTMLButtonElement = class HTMLButtonElement {};
  }
  function walk(node, visit) {
    for (const child of node.children ?? []) {
      visit(child);
      walk(child, visit);
    }
  }

  function matches(node, sel) {
    if (sel.includes("][")) {
      return sel
        .split(/(?<=\])/)
        .filter(Boolean)
        .every((part) => matches(node, part));
    }
    if (sel.startsWith("#")) {
      return node.id === sel.slice(1);
    }
    if (sel.startsWith(".")) {
      return node.className.split(/\s+/).includes(sel.slice(1));
    }
    const data = sel.match(/^\[([a-z0-9-]+)(?:=(?:"([^"]*)"|([^\]]+)))?\]$/i);
    if (data) {
      const name = data[1];
      const value = data[2] ?? data[3];
      if (name.startsWith("data-")) {
        const key = name
          .slice(5)
          .replace(/-([a-z])/g, (_, ch) => ch.toUpperCase());
        const actual = node.dataset?.[key];
        return value == null ? actual != null : String(actual) === value;
      }
      if (name === "type") {
        return node.type === value;
      }
      return false;
    }
    return node.tagName === sel.toUpperCase();
  }

  function queryAll(rootEl, sel) {
    const found = [];
    walk(rootEl, (node) => {
      if (node.nodeType === 1 && matches(node, sel)) {
        found.push(node);
      }
    });
    if (rootEl.nodeType === 1 && matches(rootEl, sel)) {
      found.unshift(rootEl);
    }
    return found;
  }

  function createElement(tagName) {
    const children = [];
    const attrs = {};
    const listeners = new Map();
    const el = {
      nodeType: 1,
      tagName: tagName.toUpperCase(),
      children,
      childNodes: children,
      hidden: false,
      dataset: {},
      className: "",
      id: "",
      lang: "",
      dir: "",
      type: "",
      disabled: false,
      parentNode: null,
      get parentElement() {
        return el.parentNode;
      },
      ownerDocument: doc,
      style: {
        setProperty(name, value) {
          this[name] = value;
        },
        getPropertyValue(name) {
          return this[name] ?? "";
        },
        removeProperty(name) {
          delete this[name];
        },
      },
      getBoundingClientRect() {
        return {
          top: 0,
          bottom: 80,
          height: 80,
          width: 320,
          left: 0,
          right: 320,
        };
      },
      scrollHeight: 0,
      clientHeight: 0,
      get classList() {
        return {
          contains: (name) => el.className.split(/\s+/).includes(name),
          add(name) {
            if (!this.contains(name)) {
              el.className = `${el.className} ${name}`.trim();
            }
          },
          remove(name) {
            el.className = el.className
              .split(/\s+/)
              .filter((part) => part && part !== name)
              .join(" ");
          },
          toggle(name, force) {
            const on = force ?? !this.contains(name);
            if (on) {
              this.add(name);
            } else {
              this.remove(name);
            }
            return on;
          },
        };
      },
      appendChild(child) {
        const from = child.parentNode?.children;
        if (from) {
          const at = from.indexOf(child);
          if (at >= 0) {
            from.splice(at, 1);
          }
        }
        child.parentNode = el;
        children.push(child);
        return child;
      },
      append(...nodes) {
        for (const node of nodes) {
          el.appendChild(node);
        }
      },
      replaceChildren(...nodes) {
        for (const child of children) {
          child.parentNode = null;
        }
        children.length = 0;
        for (const node of nodes) {
          el.appendChild(node);
        }
      },
      insertBefore(node, before) {
        const from = node.parentNode?.children;
        if (from) {
          const at = from.indexOf(node);
          if (at >= 0) {
            from.splice(at, 1);
          }
        }
        node.parentNode = el;
        const index = children.indexOf(before);
        if (index < 0) {
          children.push(node);
        } else {
          children.splice(index, 0, node);
        }
        return node;
      },
      remove() {
        const parent = el.parentNode;
        if (!parent?.children) {
          return;
        }
        const index = parent.children.indexOf(el);
        if (index >= 0) {
          parent.children.splice(index, 1);
        }
        el.parentNode = null;
      },
      setAttribute(name, value) {
        attrs[name] = String(value);
        if (name === "id") {
          el.id = String(value);
        }
        if (name === "type") {
          el.type = String(value);
        }
        if (name.startsWith("data-")) {
          const key = name
            .slice(5)
            .replace(/-([a-z])/g, (_, ch) => ch.toUpperCase());
          el.dataset[key] = String(value);
        }
      },
      getAttribute(name) {
        if (name === "id") {
          return el.id || null;
        }
        return Object.hasOwn(attrs, name) ? attrs[name] : null;
      },
      removeAttribute(name) {
        delete attrs[name];
        if (name === "disabled") {
          el.disabled = false;
        }
      },
      querySelector(sel) {
        return queryAll(el, sel)[0] ?? null;
      },
      querySelectorAll(sel) {
        return queryAll(el, sel);
      },
      closest(sel) {
        let node = el;
        while (node) {
          if (node.nodeType === 1 && matches(node, sel)) {
            return node;
          }
          node = node.parentNode;
        }
        return null;
      },
      contains(node) {
        let current = node;
        while (current) {
          if (current === el) {
            return true;
          }
          current = current.parentNode;
        }
        return false;
      },
      addEventListener(type, handler) {
        const list = listeners.get(type) ?? [];
        list.push(handler);
        listeners.set(type, list);
      },
      dispatchEvent(event) {
        const list = listeners.get(event.type) ?? [];
        for (const handler of list) {
          handler(event);
        }
        return true;
      },
      click() {
        if (el.disabled) {
          return;
        }
        el.dispatchEvent({ type: "click", target: el });
      },
    };
    if (tagName.toLowerCase() === "button") {
      Object.setPrototypeOf(el, HTMLButtonElement.prototype);
    }
    Object.defineProperty(el, "textContent", {
      get() {
        if (children.length === 0) {
          return el._text ?? "";
        }
        return children.map((child) => child.textContent ?? "").join("");
      },
      set(value) {
        children.length = 0;
        el._text = String(value);
        if (value) {
          children.push(doc.createTextNode(String(value)));
        }
      },
    });
    Object.defineProperty(el, "firstElementChild", {
      get() {
        return children.find((child) => child.nodeType === 1) ?? null;
      },
    });
    return el;
  }

  const doc = {
    documentElement: {
      dataset: { motion: "reduce" },
      hasAttribute: () => false,
    },
    defaultView: {
      matchMedia: () => ({ matches: true }),
    },
    activeElement: null,
    createElement,
    createTextNode(text) {
      return {
        nodeType: 3,
        nodeName: "#text",
        data: String(text),
        get textContent() {
          return this.data;
        },
        children: [],
      };
    },
    addEventListener() {},
    getSelection() {
      return null;
    },
  };
  doc.documentElement.ownerDocument = doc;
  return doc;
}

function createCard(doc) {
  const li = doc.createElement("li");
  li.className = "queue-item";
  const article = doc.createElement("article");
  const title = doc.createElement("h3");
  title.dataset.slot = "title";
  const body = doc.createElement("div");
  body.dataset.slot = "body";
  const source = doc.createElement("p");
  source.dataset.slot = "source";
  const sourceLabel = doc.createElement("span");
  sourceLabel.dataset.slot = "source-label";
  sourceLabel.textContent = "From {appName}";
  const sourceIcon = doc.createElement("img");
  sourceIcon.dataset.slot = "source-icon";
  source.appendChild(sourceLabel);
  source.appendChild(sourceIcon);
  const expand = doc.createElement("button");
  expand.dataset.slot = "expand";
  expand.textContent = "Show more";
  const actions = doc.createElement("div");
  actions.className = "row-actions";
  const copy = doc.createElement("button");
  copy.dataset.queueAction = "copy";
  copy.textContent = "Copy";
  const tip = doc.createElement("span");
  tip.dataset.slot = "action-tip";
  tip.hidden = true;
  actions.appendChild(copy);
  actions.appendChild(tip);
  article.appendChild(title);
  article.appendChild(body);
  article.appendChild(source);
  article.appendChild(expand);
  article.appendChild(actions);
  li.appendChild(article);
  return li;
}

function mountQueueRoot() {
  const doc = createMountDocument();
  const root = doc.createElement("main");
  root.id = "quick-panel";
  const form = doc.createElement("form");
  form.id = "composer";
  const editor = doc.createElement("div");
  editor.id = "composer-body";
  const submit = doc.createElement("button");
  submit.type = "submit";
  form.appendChild(editor);
  form.appendChild(submit);
  const sortToggle = doc.createElement("button");
  sortToggle.type = "button";
  sortToggle.id = "queue-sort-toggle";
  sortToggle.className = "queue-sort-toggle";
  sortToggle.dataset.queueSort = "newest";
  sortToggle.setAttribute("aria-label", "Show oldest first");
  sortToggle.setAttribute("aria-describedby", "queue-sort-state");
  const sortTip = doc.createElement("span");
  sortTip.id = "queue-sort-state";
  sortTip.setAttribute("data-slot", "sort-tip");
  sortTip.textContent = "Newest first";
  const sortVisual = doc.createElement("span");
  sortVisual.className = "icon-tip";
  sortVisual.setAttribute("data-slot", "sort-tip-visual");
  sortVisual.setAttribute("aria-hidden", "true");
  sortVisual.textContent = "Newest first";
  sortToggle.appendChild(sortTip);
  sortToggle.appendChild(sortVisual);
  const empty = doc.createElement("p");
  empty.id = "queue-empty";
  empty.hidden = false;
  empty.textContent = "Select text and Capture, or type here.";
  const list = doc.createElement("ul");
  list.id = "queue";
  const template = doc.createElement("template");
  template.id = "queue-item-template";
  const proto = createCard(doc);
  template.content = {
    firstElementChild: {
      cloneNode() {
        return createCard(doc);
      },
      querySelector: proto.querySelector.bind(proto),
      querySelectorAll: proto.querySelectorAll.bind(proto),
    },
    querySelector: proto.querySelector.bind(proto),
    querySelectorAll: proto.querySelectorAll.bind(proto),
  };
  const profile = doc.createElement("select");
  profile.id = "output-profile";
  profile.value = "plain";
  const actionStatus = doc.createElement("p");
  actionStatus.id = "action-status";
  const copiedMsg = doc.createElement("p");
  copiedMsg.dataset.actionMessage = "";
  copiedMsg.setAttribute("data-action-message", "");
  copiedMsg.setAttribute("data-i18n", "copy.announce.copied");
  copiedMsg.textContent = "Copied.";
  const notice = doc.createElement("div");
  notice.id = "chrome-notice";
  notice.hidden = true;
  const noticeText = doc.createElement("button");
  noticeText.id = "chrome-notice-text";
  noticeText.setAttribute("data-notice-reveal", "");
  notice.appendChild(noticeText);
  root.appendChild(form);
  root.appendChild(sortToggle);
  root.appendChild(empty);
  root.appendChild(list);
  root.appendChild(template);
  root.appendChild(profile);
  root.appendChild(actionStatus);
  root.appendChild(copiedMsg);
  root.appendChild(notice);
  return { root, list, empty, sortToggle, doc, actionStatus, notice };
}

function stubQueueInvoke(page, settings = { copy: { queueSort: "newest" } }) {
  const calls = [];
  let current = page;
  const store = settings;
  const invoke = async (cmd, args) => {
    calls.push({ cmd, args });
    if (cmd === "load_settings_v1") {
      return structuredClone(store);
    }
    if (cmd === "set_queue_sort") {
      store.copy = { ...(store.copy ?? {}), queueSort: args.sort };
      return structuredClone(store);
    }
    if (cmd === "take_notice_activation") {
      return null;
    }
    if (cmd === "copy_queue_items") {
      return "copied-text";
    }
    if (cmd === "queue_query") {
      if (typeof current === "function") {
        return current(args);
      }
      if (current?.[args?.sort] && current.newest) {
        return current[args.sort];
      }
      return current;
    }
    throw new Error(`unexpected ${cmd}`);
  };
  invoke.calls = calls;
  invoke.store = store;
  invoke.setPage = (next) => {
    current = next;
  };
  return invoke;
}

function listenStub() {
  const handlers = {};
  const previousTauri = globalThis.__TAURI__;
  const PreviousChannel = globalThis.BroadcastChannel;
  globalThis.BroadcastChannel = class {
    postMessage() {}
    close() {}
    set onmessage(_handler) {}
  };
  globalThis.__TAURI__ = {
    event: {
      listen(name, handler) {
        handlers[name] = handler;
        return Promise.resolve(() => {
          delete handlers[name];
        });
      },
      emit(name, payload) {
        handlers[name]?.({ payload });
        return Promise.resolve();
      },
    },
  };
  return {
    handlers,
    restore() {
      globalThis.__TAURI__ = previousTauri;
      globalThis.BroadcastChannel = PreviousChannel;
    },
  };
}

async function waitUntil(probe, label) {
  for (let attempt = 0; attempt < 40; attempt += 1) {
    if (probe()) {
      return;
    }
    await new Promise((resolve) => {
      setTimeout(resolve, 0);
    });
  }
  throw new Error(label);
}

function paintedIds(list) {
  return Array.from(list.children)
    .map((row) => row.dataset?.itemId)
    .filter(Boolean);
}

function enableQueueMotion(doc) {
  doc.documentElement.dataset.motion = "full";
  doc.defaultView.matchMedia = () => ({ matches: false });
  const timers = [];
  doc.defaultView.setTimeout = (fn) => {
    timers.push(fn);
    return timers.length;
  };
  doc.defaultView.clearTimeout = () => {};
  return {
    flush() {
      const queued = timers.splice(0, timers.length);
      for (const fn of queued) {
        fn?.();
      }
    },
  };
}

function rowHas(list, id, className) {
  const row = Array.from(list.children).find(
    (node) => node.dataset?.itemId === id,
  );
  return Boolean(row?.classList.contains(className));
}

test("mounting the queue paints the first page without queue-changed", async () => {
  const tauri = listenStub();
  const { root, list, empty } = mountQueueRoot();
  const invoke = stubQueueInvoke({
    items: [
      {
        id: "cursor-1",
        title: "that from Cursor",
        body: "older card",
        contentLanguage: "en",
      },
    ],
    nextCursor: null,
  });
  try {
    await bindQueueLive(root, invoke);
    assert.equal(
      invoke.calls.some((call) => call.cmd === "queue_query"),
      true,
    );
    assert.deepEqual(paintedIds(list), ["cursor-1"]);
    assert.equal(empty.hidden, true);
    assert.equal(tauri.handlers["queue-changed"] == null, false);
    assert.equal(
      invoke.calls.filter((call) => call.cmd === "queue_query").length,
      1,
    );
  } finally {
    tauri.restore();
  }
});

test("an empty first page keeps the empty state", async () => {
  const tauri = listenStub();
  const { root, list, empty } = mountQueueRoot();
  const invoke = stubQueueInvoke({ items: [], nextCursor: null });
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), []);
    assert.equal(empty.hidden, false);
  } finally {
    tauri.restore();
  }
});

test("mounting with items expands the first page once, not slide-in", async () => {
  const tauri = listenStub();
  const { root, list, doc } = mountQueueRoot();
  enableQueueMotion(doc);
  const invoke = stubQueueInvoke({
    items: [
      { id: "new", title: "just copied", body: "fresh" },
      { id: "old", title: "that from Cursor", body: "older" },
    ],
    nextCursor: null,
  });
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    assert.equal(list.classList.contains("is-sort-expand"), true);
    assert.equal(list.classList.contains("is-sort-hold"), false);
    assert.equal(list.classList.contains("is-sort-collapse"), false);
    assert.equal(rowHas(list, "new", "is-sort-lead"), true);
    for (const row of list.children) {
      assert.equal(row.classList.contains("is-sort-expand"), true);
      assert.equal(row.classList.contains("is-slide-in"), false);
      assert.equal(row.classList.contains("is-make-room"), false);
      assert.equal(row.classList.contains("is-arriving"), false);
    }
  } finally {
    tauri.restore();
  }
});

test("an empty mount with motion does not expand", async () => {
  const tauri = listenStub();
  const { root, list, empty, doc } = mountQueueRoot();
  enableQueueMotion(doc);
  const invoke = stubQueueInvoke({ items: [], nextCursor: null });
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), []);
    assert.equal(empty.hidden, false);
    assert.equal(list.classList.contains("is-sort-expand"), false);
    assert.equal(list.classList.contains("is-sort-hold"), false);
  } finally {
    tauri.restore();
  }
});

test("a later single insert still makes room after startup expand", async () => {
  const tauri = listenStub();
  const { root, list, doc } = mountQueueRoot();
  const clock = enableQueueMotion(doc);
  const invoke = stubQueueInvoke({
    items: [{ id: "old", title: "that from Cursor", body: "older" }],
    nextCursor: null,
  });
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), ["old"]);
    assert.equal(list.classList.contains("is-sort-expand"), true);
    assert.equal(rowHas(list, "old", "is-slide-in"), false);
    clock.flush();
    assert.equal(list.classList.contains("is-sort-expand"), false);
    invoke.setPage({
      items: [
        { id: "fresh", title: "just copied", body: "fresh" },
        { id: "old", title: "that from Cursor", body: "older" },
      ],
      nextCursor: null,
    });
    tauri.handlers["capture-result"]({ payload: { terminal: "saved" } });
    await waitUntil(
      () => paintedIds(list).join() === "fresh,old",
      "saved capture did not insert the new card",
    );
    assert.equal(rowHas(list, "fresh", "is-slide-in"), true);
    assert.equal(rowHas(list, "fresh", "is-sort-expand"), false);
    assert.equal(rowHas(list, "old", "is-make-room"), true);
    assert.equal(rowHas(list, "old", "is-slide-in"), false);
    assert.equal(list.classList.contains("is-sort-expand"), false);
    clock.flush();
    assert.equal(rowHas(list, "fresh", "is-ring-pulse"), true);
  } finally {
    tauri.restore();
  }
});

test("Copy does not add is-ring-pulse; a new capture still can", async () => {
  const tauri = listenStub();
  const { root, list, doc, actionStatus, notice } = mountQueueRoot();
  const clock = enableQueueMotion(doc);
  const invoke = stubQueueInvoke({
    items: [{ id: "kept", title: "that from Cursor", body: "older" }],
    nextCursor: null,
  });
  try {
    await bindQueueLive(root, invoke);
    clock.flush();
    const copy = list.querySelector('[data-queue-action="copy"]');
    assert.equal(copy instanceof HTMLButtonElement, true);
    list.dispatchEvent({ type: "click", target: copy });
    await waitUntil(
      () => invoke.calls.some((call) => call.cmd === "copy_queue_items"),
      "Copy did not invoke copy_queue_items",
    );
    assert.equal(actionStatus.textContent, "Copied.");
    assert.equal(notice.hidden, false);
    clock.flush();
    assert.equal(rowHas(list, "kept", "is-ring-pulse"), false);
    assert.equal(rowHas(list, "kept", "is-last-copied"), false);
    invoke.setPage({
      items: [
        { id: "fresh", title: "just captured", body: "fresh" },
        { id: "kept", title: "that from Cursor", body: "older" },
      ],
      nextCursor: null,
    });
    tauri.handlers["capture-result"]({ payload: { terminal: "saved" } });
    await waitUntil(
      () => paintedIds(list).join() === "fresh,kept",
      "saved capture did not insert the new card",
    );
    assert.equal(rowHas(list, "fresh", "is-slide-in"), true);
    clock.flush();
    assert.equal(rowHas(list, "fresh", "is-ring-pulse"), true);
    assert.equal(rowHas(list, "kept", "is-ring-pulse"), false);
  } finally {
    tauri.restore();
  }
});

test("reduced motion first page appears without expand", async () => {
  const tauri = listenStub();
  const { root, list } = mountQueueRoot();
  const invoke = stubQueueInvoke({
    items: [
      { id: "new", title: "just copied", body: "fresh" },
      { id: "old", title: "that from Cursor", body: "older" },
    ],
    nextCursor: null,
  });
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    assert.equal(list.classList.contains("is-sort-expand"), false);
    assert.equal(list.classList.contains("is-sort-hold"), false);
    for (const row of list.children) {
      assert.equal(row.classList.contains("is-sort-expand"), false);
      assert.equal(row.classList.contains("is-slide-in"), false);
      assert.equal(row.classList.contains("is-make-room"), false);
    }
  } finally {
    tauri.restore();
  }
});

test("title refine after first paint does not replay expand", async () => {
  const tauri = listenStub();
  const { root, list, doc } = mountQueueRoot();
  const clock = enableQueueMotion(doc);
  const invoke = stubQueueInvoke({
    items: [{ id: "kept", title: "draft title", body: "body" }],
    nextCursor: null,
  });
  try {
    await bindQueueLive(root, invoke);
    assert.equal(list.classList.contains("is-sort-expand"), true);
    clock.flush();
    assert.equal(list.classList.contains("is-sort-expand"), false);
    invoke.setPage({
      items: [{ id: "kept", title: "refined title", body: "body" }],
      nextCursor: null,
    });
    tauri.handlers["queue-changed"]({ payload: "" });
    await waitUntil(() => {
      const title = list.querySelector("[data-slot=title]");
      return title?.textContent === "refined title";
    }, "title refine did not update the painted row");
    assert.deepEqual(paintedIds(list), ["kept"]);
    assert.equal(list.classList.contains("is-sort-expand"), false);
    assert.equal(rowHas(list, "kept", "is-sort-expand"), false);
    assert.equal(rowHas(list, "kept", "is-slide-in"), false);
  } finally {
    tauri.restore();
  }
});

test("a later capture still live-updates the loaded page", async () => {
  const tauri = listenStub();
  const { root, list, empty } = mountQueueRoot();
  const invoke = stubQueueInvoke({
    items: [{ id: "old", title: "that from Cursor", body: "older" }],
    nextCursor: null,
  });
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), ["old"]);
    invoke.setPage({
      items: [
        { id: "new", title: "just copied", body: "fresh" },
        { id: "old", title: "that from Cursor", body: "older" },
      ],
      nextCursor: null,
    });
    tauri.handlers["queue-changed"]({ payload: "new" });
    await waitUntil(
      () => paintedIds(list).join() === "new,old",
      "queue-changed did not paint the live page",
    );
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    assert.equal(empty.hidden, true);
  } finally {
    tauri.restore();
  }
});

test("a failed first query retries when locale is applied", async () => {
  const tauri = listenStub();
  const { root, list, empty } = mountQueueRoot();
  let attempts = 0;
  const invoke = stubQueueInvoke(() => {
    attempts += 1;
    if (attempts === 1) {
      throw new Error("invoke_unavailable");
    }
    return {
      items: [{ id: "stored", title: "that from Cursor", body: "older" }],
      nextCursor: null,
    };
  });
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), []);
    assert.equal(empty.hidden, false);
    root.dispatchEvent({ type: LOCALE_APPLIED_EVENT });
    await waitUntil(
      () => paintedIds(list).join() === "stored",
      "locale retry did not paint the first page",
    );
    assert.deepEqual(paintedIds(list), ["stored"]);
    assert.equal(empty.hidden, true);
  } finally {
    tauri.restore();
  }
});

test("clicking the sort toggle writes oldest and reloads the first page", async () => {
  const tauri = listenStub();
  const { root, list, sortToggle } = mountQueueRoot();
  const invoke = stubQueueInvoke({
    newest: {
      items: [
        { id: "new", title: "just copied", body: "fresh" },
        { id: "old", title: "that from Cursor", body: "older" },
      ],
      nextCursor: null,
    },
    oldest: {
      items: [
        { id: "old", title: "that from Cursor", body: "older" },
        { id: "new", title: "just copied", body: "fresh" },
      ],
      nextCursor: null,
    },
  });
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    assert.equal(sortToggle.dataset.queueSort, "newest");
    assert.equal(sortToggle.getAttribute("aria-label"), "Show oldest first");
    sortToggle.click();
    await waitUntil(
      () =>
        paintedIds(list).join() === "old,new" &&
        sortToggle.dataset.queueSort === "oldest",
      "sort toggle did not persist oldest and reload",
    );
    assert.deepEqual(paintedIds(list), ["old", "new"]);
    assert.equal(invoke.store.copy.queueSort, "oldest");
    assert.equal(
      invoke.calls.some(
        (call) => call.cmd === "set_queue_sort" && call.args.sort === "oldest",
      ),
      true,
    );
    assert.equal(
      invoke.calls.some((call) => call.cmd === "save_settings_v1"),
      false,
    );
    assert.equal(
      invoke.calls.filter(
        (call) => call.cmd === "queue_query" && call.args?.sort === "oldest",
      ).length > 0,
      true,
    );
    assert.equal(sortToggle.getAttribute("aria-label"), "Show newest first");
    assert.equal(
      sortToggle.getAttribute("data-i18n-aria-label"),
      "queue.sort.showNewest",
    );
    assert.equal(
      sortToggle.getAttribute("aria-describedby"),
      "queue-sort-state",
    );
    assert.equal(
      sortToggle.querySelector("[data-slot=sort-tip]").textContent,
      "Oldest first",
    );
    assert.equal(list.classList.contains("is-sort-collapse"), false);
    assert.equal(list.classList.contains("is-sort-expand"), false);
    assert.equal(list.classList.contains("is-sort-hold"), false);
  } finally {
    tauri.restore();
  }
});

function sortPages() {
  return {
    newest: {
      items: [
        { id: "new", title: "just copied", body: "fresh" },
        { id: "old", title: "that from Cursor", body: "older" },
      ],
      nextCursor: null,
    },
    oldest: {
      items: [
        { id: "old", title: "that from Cursor", body: "older" },
        { id: "new", title: "just copied", body: "fresh" },
      ],
      nextCursor: null,
    },
  };
}

function countReplaces(list) {
  let replaces = 0;
  const original = list.replaceChildren.bind(list);
  list.replaceChildren = (...nodes) => {
    replaces += 1;
    return original(...nodes);
  };
  return {
    get count() {
      return replaces;
    },
  };
}

test("sort toggle with motion collapses then expands the loaded page", async () => {
  const tauri = listenStub();
  const { root, list, sortToggle, doc } = mountQueueRoot();
  doc.documentElement.dataset.motion = "full";
  doc.defaultView.matchMedia = () => ({ matches: false });
  const timers = [];
  doc.defaultView.setTimeout = (fn) => {
    timers.push(fn);
    return timers.length;
  };
  const invoke = stubQueueInvoke(sortPages());
  const replaces = countReplaces(list);
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    const replacesAfterLoad = replaces.count;
    timers.length = 0;
    sortToggle.click();
    await waitUntil(
      () => list.classList.contains("is-sort-collapse"),
      "sort collapse did not start",
    );
    assert.equal(list.classList.contains("is-sort-expand"), false);
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    assert.equal(replaces.count, replacesAfterLoad);
    const collapseTimer = timers.shift();
    collapseTimer?.();
    await waitUntil(
      () =>
        paintedIds(list).join() === "old,new" &&
        list.classList.contains("is-sort-expand"),
      "sort expand did not follow collapse",
    );
    assert.equal(replaces.count, replacesAfterLoad + 1);
    assert.equal(list.classList.contains("is-sort-collapse"), false);
    assert.equal(list.classList.contains("is-sort-expand"), true);
    assert.equal(list.classList.contains("is-sort-hold"), false);
    assert.equal(sortToggle.dataset.queueSort, "oldest");
    assert.equal(
      sortToggle.querySelector("[data-slot=sort-tip]").textContent,
      "Oldest first",
    );
  } finally {
    tauri.restore();
  }
});

test("reduced motion sort replaces the first page once", async () => {
  const tauri = listenStub();
  const { root, list, sortToggle, doc } = mountQueueRoot();
  doc.documentElement.dataset.motion = "reduce";
  const invoke = stubQueueInvoke(sortPages());
  const replaces = countReplaces(list);
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    const replacesAfterLoad = replaces.count;
    sortToggle.click();
    await waitUntil(
      () => paintedIds(list).join() === "old,new",
      "reduced motion sort did not replace the first page",
    );
    assert.equal(replaces.count, replacesAfterLoad + 1);
    assert.equal(list.classList.contains("is-sort-collapse"), false);
    assert.equal(list.classList.contains("is-sort-expand"), false);
    assert.equal(list.classList.contains("is-sort-hold"), false);
    assert.equal(sortToggle.dataset.queueSort, "oldest");
  } finally {
    tauri.restore();
  }
});

test("a second sort click during collapse keeps the latest order", async () => {
  const tauri = listenStub();
  const { root, list, sortToggle, doc } = mountQueueRoot();
  doc.documentElement.dataset.motion = "full";
  doc.defaultView.matchMedia = () => ({ matches: false });
  const timers = [];
  doc.defaultView.setTimeout = (fn) => {
    timers.push(fn);
    return timers.length;
  };
  const invoke = stubQueueInvoke(sortPages());
  try {
    await bindQueueLive(root, invoke);
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    timers.length = 0;
    sortToggle.click();
    await waitUntil(
      () => list.classList.contains("is-sort-collapse"),
      "first sort collapse did not start",
    );
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    sortToggle.click();
    const collapseTimer = timers.shift();
    collapseTimer?.();
    await waitUntil(
      () =>
        paintedIds(list).join() === "new,old" &&
        list.classList.contains("is-sort-expand"),
      "second sort did not expand the latest first page",
    );
    assert.equal(list.classList.contains("is-sort-collapse"), false);
    assert.equal(list.classList.contains("is-sort-hold"), false);
    assert.equal(sortToggle.dataset.queueSort, "newest");
    while (timers.length) {
      timers.shift()?.();
    }
    assert.equal(list.classList.contains("is-sort-expand"), false);
    assert.equal(list.classList.contains("is-sort-collapse"), false);
    assert.equal(list.classList.contains("is-sort-hold"), false);
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    for (const row of list.children) {
      assert.equal(row.classList.contains("is-sort-collapse"), false);
      assert.equal(row.classList.contains("is-sort-expand"), false);
      assert.equal(row.classList.contains("is-sort-hold"), false);
    }
  } finally {
    tauri.restore();
  }
});

test("a second queue-sort-changed during collapse does not swap early", async () => {
  const tauri = listenStub();
  const { root, list, sortToggle, doc } = mountQueueRoot();
  doc.documentElement.dataset.motion = "full";
  doc.defaultView.matchMedia = () => ({ matches: false });
  const timers = [];
  doc.defaultView.setTimeout = (fn) => {
    timers.push(fn);
    return timers.length;
  };
  const invoke = stubQueueInvoke(sortPages());
  const replaces = countReplaces(list);
  try {
    await bindQueueLive(root, invoke);
    const replacesAfterLoad = replaces.count;
    timers.length = 0;
    sortToggle.click();
    await waitUntil(
      () => list.classList.contains("is-sort-collapse"),
      "sort collapse did not start",
    );
    tauri.handlers["queue-sort-changed"]?.({ payload: { sort: "oldest" } });
    assert.deepEqual(paintedIds(list), ["new", "old"]);
    assert.equal(replaces.count, replacesAfterLoad);
    assert.equal(list.classList.contains("is-sort-collapse"), true);
    const collapseTimer = timers.shift();
    collapseTimer?.();
    await waitUntil(
      () =>
        paintedIds(list).join() === "old,new" &&
        list.classList.contains("is-sort-expand"),
      "coalesced sort did not expand after collapse",
    );
    assert.equal(replaces.count, replacesAfterLoad + 1);
  } finally {
    tauri.restore();
  }
});
