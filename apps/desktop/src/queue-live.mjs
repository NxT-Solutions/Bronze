import { catalogMessage, LOCALE_APPLIED_EVENT } from "./apply-locale.mjs";
import {
  applyActionStatus,
  bindChromeNotice,
  bindIconTips,
  bindOverflowDismiss,
  closeOverflowMenus,
  hideChromeNotice,
  openEditSheet,
  runBusy,
  showChromeNotice,
} from "./control.mjs";
import { formatCaptureSource } from "./item-view.mjs";
import { serializeComposerDom } from "./markdown-body.mjs";
import {
  applyQueueItemMutation,
  createQueueRenderer,
} from "./queue-motion.mjs";
import {
  findQueueCard,
  focusQueueCard,
  isNearLoadedStart,
  markLastCopied,
  motionForReveal,
  planNewItemFollow,
  scrollQueueCard,
  shouldLoadNextOnKey,
} from "./queue-reveal.mjs";
import {
  listenQueueSortChanged,
  parseQueueSort,
  queueListArgs,
} from "./queue-sort.mjs";
import { tauriInvoke } from "./tauri-bridge.mjs";

export { formatCaptureSource, serializeComposerDom };

export const QUEUE_PAGE_SIZE = 20;

export function mergeQueueHead(loaded, head) {
  const seen = new Set();
  const next = [];
  for (const item of head ?? []) {
    if (!item?.id || seen.has(item.id)) {
      continue;
    }
    seen.add(item.id);
    next.push(item);
  }
  for (const item of loaded ?? []) {
    if (!item?.id || seen.has(item.id)) {
      continue;
    }
    seen.add(item.id);
    next.push(item);
  }
  return next;
}

export function appendQueuePage(loaded, pageItems) {
  const incoming = new Map();
  for (const item of pageItems ?? []) {
    if (item?.id) {
      incoming.set(item.id, item);
    }
  }
  const next = (loaded ?? []).map((item) => incoming.get(item.id) ?? item);
  const seen = new Set(next.map((item) => item.id));
  for (const item of pageItems ?? []) {
    if (!item?.id || seen.has(item.id)) {
      continue;
    }
    seen.add(item.id);
    next.push(item);
  }
  return next;
}

export function applyQueuePage(state, page, mode) {
  const items = state?.items ?? [];
  const incoming = Array.isArray(page?.items) ? page.items : [];
  const pageCursor = page?.nextCursor ?? null;
  if (mode === "more") {
    return {
      items: appendQueuePage(items, incoming),
      nextCursor: pageCursor,
      anchorCursor: state?.nextCursor ?? null,
    };
  }
  if (mode === "tail") {
    return {
      items: appendQueuePage(items, incoming),
      nextCursor: pageCursor ?? state?.nextCursor ?? null,
      anchorCursor: state?.anchorCursor ?? null,
    };
  }
  return {
    items: mergeQueueHead(items, incoming),
    nextCursor: items.length === 0 ? pageCursor : (state?.nextCursor ?? null),
    anchorCursor: items.length === 0 ? null : (state?.anchorCursor ?? null),
  };
}

export function removeQueueItem(state, id) {
  return {
    items: (state?.items ?? []).filter((item) => item.id !== id),
    nextCursor: state?.nextCursor ?? null,
    anchorCursor: state?.anchorCursor ?? null,
  };
}

export function queueFocusAtEnd(rows, target) {
  if (!target || !rows?.length) {
    return false;
  }
  const last = rows[rows.length - 1];
  if (last === target) {
    return true;
  }
  return typeof last?.contains === "function" && last.contains(target);
}

const FORMAT_TAGS = {
  strong: new Set(["strong", "b"]),
  em: new Set(["em", "i"]),
};

export function composerFormatAction({ collapsed, alreadyOn }) {
  if (alreadyOn && collapsed) {
    return "exit";
  }
  if (alreadyOn) {
    return "unwrap";
  }
  if (collapsed) {
    return "insert";
  }
  return "wrap";
}

function formatAncestor(node, tagName, root) {
  const match = FORMAT_TAGS[tagName] ?? new Set([tagName]);
  let el = node?.nodeType === 1 ? node : node?.parentElement;
  while (el && el !== root && root.contains(el)) {
    if (match.has(el.tagName.toLowerCase())) {
      return el;
    }
    el = el.parentElement;
  }
  return null;
}

function unwrapElement(el) {
  const parent = el.parentNode;
  if (!parent) {
    return;
  }
  while (el.firstChild) {
    parent.insertBefore(el.firstChild, el);
  }
  parent.removeChild(el);
}

function markHasVisibleText(el) {
  return (el.textContent ?? "").replaceAll("\u200B", "").length > 0;
}

function placeCaret(sel, doc, node, offset) {
  const range = doc.createRange();
  range.setStart(node, offset);
  range.collapse(true);
  sel.removeAllRanges();
  sel.addRange(range);
}

function ensureComposerCaret(editor) {
  const doc = editor.ownerDocument;
  editor.focus?.();
  const sel = doc.getSelection();
  if (
    sel?.rangeCount &&
    sel.anchorNode &&
    (sel.anchorNode === editor || editor.contains(sel.anchorNode))
  ) {
    return sel;
  }
  if (!sel || !doc.createRange) {
    return null;
  }
  const range = doc.createRange();
  range.selectNodeContents(editor);
  range.collapse(false);
  sel.removeAllRanges();
  sel.addRange(range);
  return sel;
}

function insertTypingMark(editor, tagName, sel) {
  const doc = editor.ownerDocument;
  const range = sel.getRangeAt(0);
  const el = doc.createElement(tagName);
  const mark = doc.createTextNode("\u200B");
  el.appendChild(mark);
  range.insertNode(el);
  placeCaret(sel, doc, mark, 1);
}

function exitTypingMark(el, sel) {
  const doc = el.ownerDocument;
  const parent = el.parentNode;
  if (!parent || !sel.rangeCount) {
    return;
  }
  const caret = sel.getRangeAt(0);
  const after = doc.createRange();
  after.setStart(caret.startContainer, caret.startOffset);
  after.setEnd(el, el.childNodes.length);
  const tail = after.extractContents();
  const nextSibling = el.nextSibling;
  if (markHasVisibleText(tail)) {
    const copy = el.cloneNode(false);
    copy.appendChild(tail);
    parent.insertBefore(copy, nextSibling);
  }
  if (!markHasVisibleText(el)) {
    parent.removeChild(el);
  }
  const next = doc.createRange();
  if (el.parentNode) {
    next.setStartAfter(el);
  } else if (nextSibling?.parentNode) {
    next.setStartBefore(nextSibling);
  } else {
    next.selectNodeContents(parent);
    next.collapse(false);
  }
  next.collapse(true);
  sel.removeAllRanges();
  sel.addRange(next);
}

export function applyComposerFormat(editor, tagName) {
  if (
    !editor ||
    (tagName !== "strong" && tagName !== "em") ||
    !editor.ownerDocument?.createElement
  ) {
    return;
  }
  const sel = ensureComposerCaret(editor);
  if (!sel || sel.rangeCount === 0) {
    return;
  }
  const existing = formatAncestor(sel.anchorNode, tagName, editor);
  const action = composerFormatAction({
    collapsed: sel.isCollapsed,
    alreadyOn: Boolean(existing),
  });
  if (action === "exit" && existing) {
    exitTypingMark(existing, sel);
    return;
  }
  if (action === "unwrap" && existing) {
    unwrapElement(existing);
    return;
  }
  if (action === "insert") {
    insertTypingMark(editor, tagName, sel);
    return;
  }
  const range = sel.getRangeAt(0);
  const el = editor.ownerDocument.createElement(tagName);
  el.appendChild(range.extractContents());
  range.insertNode(el);
  sel.removeAllRanges();
  const next = editor.ownerDocument.createRange();
  next.selectNodeContents(el);
  sel.addRange(next);
}

export function wrapComposerSelection(root, tagName) {
  applyComposerFormat(root, tagName);
}

function closestTag(node, root, tags) {
  let el = node?.nodeType === 1 ? node : node?.parentElement;
  while (el && el !== root && root.contains(el)) {
    if (tags.has(el.tagName.toLowerCase())) {
      return el;
    }
    el = el.parentElement;
  }
  return null;
}

function placeCaretIn(sel, doc, node) {
  const range = doc.createRange();
  if (node.nodeType === 3) {
    range.setStart(node, node.data?.length ?? 0);
  } else {
    range.selectNodeContents(node);
    range.collapse(false);
  }
  sel.removeAllRanges();
  sel.addRange(range);
}

function unwrapList(list) {
  const parent = list.parentNode;
  const doc = list.ownerDocument;
  const first = [];
  for (const li of Array.from(list.children)) {
    if (li.tagName !== "LI") {
      continue;
    }
    const block = doc.createElement("div");
    while (li.firstChild) {
      block.appendChild(li.firstChild);
    }
    parent.insertBefore(block, list);
    first.push(block);
  }
  parent.removeChild(list);
  return first[0] ?? null;
}

function wrapCurrentInList(editor, sel, listTag) {
  const doc = editor.ownerDocument;
  const list = doc.createElement(listTag);
  const li = doc.createElement("li");
  const block = closestTag(sel.anchorNode, editor, new Set(["div", "p"]));
  if (block && block.parentNode === editor) {
    while (block.firstChild) {
      li.appendChild(block.firstChild);
    }
    if (!markHasVisibleText(li)) {
      li.appendChild(doc.createTextNode("\u200B"));
    }
    list.appendChild(li);
    block.parentNode.insertBefore(list, block);
    block.parentNode.removeChild(block);
  } else {
    while (editor.firstChild) {
      li.appendChild(editor.firstChild);
    }
    if (!markHasVisibleText(li)) {
      li.appendChild(doc.createTextNode("\u200B"));
    }
    list.appendChild(li);
    editor.appendChild(list);
  }
  placeCaretIn(sel, doc, li.firstChild ?? li);
}

export function applyComposerList(editor, listTag) {
  if (!editor || (listTag !== "ul" && listTag !== "ol")) {
    return;
  }
  const sel = ensureComposerCaret(editor);
  if (!sel || sel.rangeCount === 0) {
    return;
  }
  const existing = closestTag(sel.anchorNode, editor, new Set(["ul", "ol"]));
  if (existing) {
    if (existing.tagName.toLowerCase() === listTag) {
      const block = unwrapList(existing);
      if (block) {
        placeCaretIn(sel, editor.ownerDocument, block.firstChild ?? block);
      }
      return;
    }
    const next = editor.ownerDocument.createElement(listTag);
    while (existing.firstChild) {
      next.appendChild(existing.firstChild);
    }
    existing.parentNode.insertBefore(next, existing);
    existing.parentNode.removeChild(existing);
    return;
  }
  wrapCurrentInList(editor, sel, listTag);
}

function exitListAtItem(li) {
  const list = li.parentElement;
  const parent = list?.parentNode;
  const doc = li.ownerDocument;
  if (!list || !parent) {
    return;
  }
  const following = [];
  let sib = li.nextSibling;
  while (sib) {
    const next = sib.nextSibling;
    following.push(sib);
    sib = next;
  }
  list.removeChild(li);
  const block = doc.createElement("div");
  block.appendChild(doc.createTextNode("\u200B"));
  if (following.length > 0) {
    const rest = doc.createElement(list.tagName);
    for (const item of following) {
      rest.appendChild(item);
    }
    parent.insertBefore(block, list.nextSibling);
    parent.insertBefore(rest, block.nextSibling);
  } else {
    parent.insertBefore(block, list.nextSibling);
  }
  if (!list.querySelector("li")) {
    parent.removeChild(list);
  }
  return block;
}

export function applyComposerCommand(editor, tag) {
  if (tag === "ul" || tag === "ol") {
    applyComposerList(editor, tag);
    return;
  }
  applyComposerFormat(editor, tag);
}

export function composerHotkey(event) {
  if (event.isComposing) {
    return null;
  }
  if (composerShouldSubmit(event)) {
    return "submit";
  }
  const meta = Boolean(event.metaKey || event.ctrlKey);
  const key = typeof event.key === "string" ? event.key.toLowerCase() : "";
  if (meta && !event.shiftKey && !event.altKey) {
    if (key === "b") {
      return "strong";
    }
    if (key === "i") {
      return "em";
    }
  }
  if (meta && event.shiftKey && !event.altKey) {
    if (event.code === "Digit8" || key === "8" || key === "*") {
      return "ul";
    }
    if (event.code === "Digit7" || key === "7") {
      return "ol";
    }
  }
  if (event.key === "Enter" && !meta && !event.shiftKey && !event.altKey) {
    return "list-break";
  }
  return null;
}

export function syncComposerEmpty(el) {
  if (!el?.classList) {
    return;
  }
  el.classList.toggle("is-empty", serializeComposerDom(el).length === 0);
}

function syncFormatPressed(root, editor) {
  const sel = editor.ownerDocument?.getSelection?.();
  const anchor = sel?.anchorNode;
  const list = anchor
    ? closestTag(anchor, editor, new Set(["ul", "ol"]))
    : null;
  for (const button of root.querySelectorAll("[data-composer-format]")) {
    const tag = button.getAttribute("data-composer-format");
    const on =
      tag === "ul" || tag === "ol"
        ? list?.tagName.toLowerCase() === tag
        : Boolean(anchor && formatAncestor(anchor, tag, editor));
    button.setAttribute("aria-pressed", on ? "true" : "false");
  }
}

export const QUE_007_COMPLETE = false;

export function composerIsMultiline(body) {
  return String(body ?? "").includes("\n");
}

export function composerSubmitLabelKey(body) {
  return composerIsMultiline(body)
    ? "composer.add.submit.chord"
    : "composer.add.submit";
}

export function applyComposerSubmitLabel(button, body) {
  if (!button) {
    return;
  }
  const key = composerSubmitLabelKey(body);
  button.setAttribute("data-i18n", key);
  const label = catalogMessage(key);
  if (label) {
    button.textContent = label;
  }
}

export function composerShouldSubmit(event) {
  if (event.type === "submit") {
    return true;
  }
  if (event.key !== "Enter") {
    return false;
  }
  if (event.isComposing) {
    return false;
  }
  return Boolean(
    event.shiftKey && !event.metaKey && !event.ctrlKey && !event.altKey,
  );
}

export function listenQueueChanged(handler) {
  const listen = globalThis.__TAURI__?.event?.listen;
  if (typeof listen === "function") {
    return listen("queue-changed", handler);
  }
  return Promise.resolve(null);
}

export function listenCaptureResult(handler) {
  const listen = globalThis.__TAURI__?.event?.listen;
  if (typeof listen === "function") {
    return listen("capture-result", handler);
  }
  return Promise.resolve(null);
}

export function captureFeedbackKey(result) {
  if (!result || typeof result !== "object") {
    return "capture.announce.failed";
  }
  if (result.terminal === "saved") {
    return "capture.announce.saved";
  }
  if (result.reason === "accessibility") {
    return "capture.announce.denied";
  }
  if (result.reason === "protected") {
    return "capture.announce.protected";
  }
  if (result.reason === "no_selection") {
    return "capture.announce.rejected";
  }
  if (result.reason === "app_excluded") {
    return "capture.announce.excluded";
  }
  return "capture.announce.failed";
}

export function applyCaptureResult(root, result) {
  const status = root.querySelector("#capture-status");
  if (!status) {
    return;
  }
  const key = captureFeedbackKey(result);
  const source = root.querySelector(
    `[data-capture-message][data-i18n="${key}"]`,
  );
  const text = source?.textContent?.trim() ?? "";
  status.textContent = text;
  status.hidden = text.length === 0;
  showChromeNotice(root, text, result?.terminal === "saved" ? "ok" : "failed");
}

export function queueMoveDisabled(index, count) {
  return {
    moveUp: count < 1 || index <= 0,
    moveDown: count < 1 || index >= count - 1,
  };
}

function setQueueMoveDisabled(button, disabled) {
  if (!button || !("disabled" in button)) {
    return;
  }
  button.disabled = disabled;
}

export function applyQueueMoveAvailability(row, index, count) {
  const flags = queueMoveDisabled(index, count);
  setQueueMoveDisabled(
    row?.querySelector?.('[data-queue-action="moveUp"]'),
    flags.moveUp,
  );
  setQueueMoveDisabled(
    row?.querySelector?.('[data-queue-action="moveDown"]'),
    flags.moveDown,
  );
}

function queueItemRows(list) {
  if (!list) {
    return [];
  }
  if (list.children) {
    return Array.from(list.children);
  }
  if (typeof list.querySelectorAll === "function") {
    return Array.from(list.querySelectorAll(".queue-item"));
  }
  return [];
}

export function syncQueueMoveAvailability(list) {
  const rows = queueItemRows(list);
  const count = rows.length;
  for (const [index, row] of rows.entries()) {
    applyQueueMoveAvailability(row, index, count);
  }
}

function observeQueueOrder(list) {
  if (!list || typeof MutationObserver !== "function") {
    return;
  }
  const observer = new MutationObserver(() => {
    syncQueueMoveAvailability(list);
  });
  observer.observe(list, { childList: true });
}

const queueRenderer = createQueueRenderer({
  queueItemRows,
  syncMoveAvailability: syncQueueMoveAvailability,
});

export function renderQueueItems(list, items, template, options = {}) {
  return queueRenderer.renderQueueItems(list, items, template, options);
}

export async function bindQueueLive(root = document, invokeFn = tauriInvoke) {
  const list = root.querySelector("#queue");
  const form = root.querySelector("#composer");
  const editor = root.querySelector("#composer-body");
  const error = root.querySelector("#composer-error");
  const empty = root.querySelector("#queue-empty");
  const template = root.querySelector("#queue-item-template");
  const profile = root.querySelector("#output-profile");
  if (!list || !form || !editor || !template) {
    return;
  }

  bindOverflowDismiss(root);
  bindIconTips(root);
  bindChromeNotice(root);
  observeQueueOrder(list);
  syncComposerEmpty(editor);
  const submit = form.querySelector("[type=submit]");
  function syncSubmitLabel() {
    applyComposerSubmitLabel(submit, serializeComposerDom(editor));
  }
  syncSubmitLabel();

  const moreStatus = root.querySelector("#queue-more-status");
  const scroller = list.closest?.("#quick-panel") ?? null;
  let state = { items: [], nextCursor: null, anchorCursor: null };
  let loadingMore = false;
  let refreshGen = 0;
  let activeSort = "newest";
  let lastCopiedId = "";

  function focusedQueueControl() {
    const active = list.ownerDocument?.activeElement;
    if (
      !active ||
      typeof list.contains !== "function" ||
      !list.contains(active)
    ) {
      return null;
    }
    const row = active.closest?.("[data-item-id], li");
    return {
      itemId: active.dataset?.itemId || row?.dataset?.itemId,
      action: active.dataset?.queueAction,
    };
  }

  function restoreQueueFocus(saved) {
    if (!saved?.itemId || typeof list.querySelectorAll !== "function") {
      return;
    }
    for (const button of list.querySelectorAll("[data-queue-action]")) {
      if (
        button.dataset?.itemId === saved.itemId &&
        button.dataset?.queueAction === saved.action
      ) {
        button.focus?.();
        return;
      }
    }
  }

  async function queryPage(cursor) {
    const paging = queueListArgs(activeSort, cursor);
    const args = {
      filter: "overview",
      limit: QUEUE_PAGE_SIZE,
      sort: paging.sort,
    };
    if (paging.cursor) {
      args.cursor = paging.cursor;
    }
    return invokeFn("queue_query", args);
  }

  async function paint(opts) {
    const saved = focusedQueueControl();
    try {
      await renderQueueItems(list, state.items, template, opts);
    } catch {
      queueRenderer.paintQueueItems(list, state.items, template);
    }
    restoreQueueFocus(saved);
    markLastCopied(list, lastCopiedId);
    if (empty) {
      empty.hidden = state.items.length > 0;
    }
  }

  async function syncHead() {
    const head = await queryPage(null);
    let next = applyQueuePage(state, head, "head");
    if (next.nextCursor == null && next.anchorCursor) {
      const tail = await queryPage(next.anchorCursor);
      next = applyQueuePage(next, tail, "tail");
    }
    state = next;
  }

  async function reloadWindow(count) {
    let cursor = null;
    let items = [];
    let nextCursor = null;
    let anchorCursor = null;
    const target = Math.max(count, 1);
    while (items.length < target) {
      const page = await queryPage(cursor);
      const before = items.length;
      items = appendQueuePage(items, page?.items ?? []);
      anchorCursor = cursor;
      nextCursor = page?.nextCursor ?? null;
      if (!nextCursor || items.length === before) {
        break;
      }
      cursor = nextCursor;
    }
    state = { items, nextCursor, anchorCursor };
  }

  async function refresh(opts = {}) {
    const gen = ++refreshGen;
    const nearStart = isNearLoadedStart(scroller);
    const prevIds = state.items.map((item) => item.id);
    let sort = "newest";
    try {
      const settings = await invokeFn("load_settings_v1");
      sort = parseQueueSort(settings?.copy?.queueSort);
    } catch {
      sort = "newest";
    }
    if (opts.resetPage || sort !== activeSort) {
      activeSort = sort;
      state = { items: [], nextCursor: null, anchorCursor: null };
    }
    const action = opts.action;
    if (action === "complete" || action === "skip" || action === "trash") {
      state = removeQueueItem(state, opts.id);
    } else if (action === "moveUp" || action === "moveDown") {
      await reloadWindow(state.items.length);
    } else if (!(action === "replace" && !opts.item)) {
      if (opts.item?.id) {
        state = {
          ...state,
          items: state.items.map((row) =>
            row.id === opts.item.id ? { ...row, ...opts.item } : row,
          ),
        };
      }
      await syncHead();
    }
    if (gen !== refreshGen) {
      return;
    }
    await paint(opts);
    if (!opts.followNew) {
      return;
    }
    const plan = planNewItemFollow({
      sort: activeSort,
      nearStart,
      prevIds,
      nextIds: state.items.map((item) => item.id),
    });
    if (plan.scrollId) {
      scrollQueueCard(findQueueCard(list, plan.scrollId), {
        motion: motionForReveal(root),
      });
    }
  }

  async function revealCopied(id) {
    if (!id) {
      return;
    }
    lastCopiedId = id;
    let card = findQueueCard(list, id);
    if (!card) {
      const page = await invokeFn("queue_page_for_item", {
        id,
        sort: activeSort,
      });
      state = {
        items: page?.items ?? [],
        nextCursor: page?.nextCursor ?? null,
        anchorCursor: null,
      };
      await paint({ action: "replace" });
      card = findQueueCard(list, id);
    } else {
      markLastCopied(list, id);
    }
    scrollQueueCard(card, { motion: motionForReveal(root) });
    focusQueueCard(card);
  }

  async function loadMore() {
    if (!state.nextCursor || loadingMore) {
      return;
    }
    loadingMore = true;
    if (moreStatus) {
      moreStatus.hidden = false;
    }
    const gen = refreshGen;
    try {
      const page = await queryPage(state.nextCursor);
      if (gen !== refreshGen) {
        return;
      }
      state = applyQueuePage(state, page, "more");
      await paint({ action: "insert" });
    } finally {
      loadingMore = false;
      if (moreStatus) {
        moreStatus.hidden = true;
      }
    }
  }

  async function addFromComposer() {
    try {
      await invokeFn("add_composer_item", {
        body: serializeComposerDom(editor),
      });
      editor.replaceChildren();
      syncComposerEmpty(editor);
      syncSubmitLabel();
      if (error) {
        error.hidden = true;
      }
      hideChromeNotice(root);
      await refresh({ action: "insert", followNew: true });
    } catch {
      if (error) {
        error.hidden = true;
        showChromeNotice(root, error.textContent?.trim() ?? "", "failed");
      }
    }
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (composerShouldSubmit(event)) {
      const submit = form.querySelector("[type=submit]");
      runBusy(submit, addFromComposer);
    }
  });
  form.addEventListener("mousedown", (event) => {
    if (event.target.closest("[data-composer-format]")) {
      event.preventDefault();
    }
  });
  form.addEventListener("click", (event) => {
    const button = event.target.closest("[data-composer-format]");
    if (!button) {
      return;
    }
    const tag = button.getAttribute("data-composer-format");
    applyComposerCommand(editor, tag);
    syncComposerEmpty(editor);
    syncSubmitLabel();
    syncFormatPressed(root, editor);
  });
  form.addEventListener("keydown", (event) => {
    const action = composerHotkey(event);
    if (action === "submit") {
      event.preventDefault();
      const submit = form.querySelector("[type=submit]");
      runBusy(submit, addFromComposer);
      return;
    }
    if (
      action === "strong" ||
      action === "em" ||
      action === "ul" ||
      action === "ol"
    ) {
      event.preventDefault();
      applyComposerCommand(editor, action);
      syncComposerEmpty(editor);
      syncSubmitLabel();
      syncFormatPressed(root, editor);
      return;
    }
    if (action === "list-break") {
      const sel = editor.ownerDocument?.getSelection?.();
      const li = sel?.anchorNode
        ? closestTag(sel.anchorNode, editor, new Set(["li"]))
        : null;
      if (li && !markHasVisibleText(li)) {
        event.preventDefault();
        const block = exitListAtItem(li);
        if (block) {
          placeCaretIn(sel, editor.ownerDocument, block.firstChild ?? block);
        }
        syncComposerEmpty(editor);
        syncSubmitLabel();
        syncFormatPressed(root, editor);
      }
    }
  });
  editor.addEventListener("input", () => {
    syncComposerEmpty(editor);
    syncSubmitLabel();
  });
  editor.addEventListener("paste", (event) => {
    event.preventDefault();
    const text = event.clipboardData?.getData("text/plain") ?? "";
    const doc = editor.ownerDocument;
    const sel = doc?.getSelection?.();
    if (!sel || sel.rangeCount === 0 || !text) {
      return;
    }
    sel.deleteFromDocument();
    const node = doc.createTextNode(text);
    const range = sel.getRangeAt(0);
    range.insertNode(node);
    range.setStartAfter(node);
    range.collapse(true);
    sel.removeAllRanges();
    sel.addRange(range);
    syncComposerEmpty(editor);
    syncSubmitLabel();
  });
  editor.ownerDocument?.addEventListener("selectionchange", () => {
    syncFormatPressed(root, editor);
  });

  list.addEventListener("click", async (event) => {
    const button = event.target.closest("[data-queue-action]");
    if (!(button instanceof HTMLButtonElement) || button.disabled) {
      return;
    }
    const id = button.dataset.itemId;
    const action = button.dataset.queueAction;
    if (!id || !action) {
      return;
    }
    if (action === "moveUp" || action === "moveDown") {
      const row = button.closest("li");
      const rows = queueItemRows(list);
      const index = rows.indexOf(row);
      if (queueMoveDisabled(index, rows.length)[action]) {
        return;
      }
    }
    await runBusy(button, async () => {
      if (action === "copy") {
        try {
          await invokeFn("copy_queue_items", {
            itemIds: [id],
            profile: profile?.value ?? "plain",
          });
          applyActionStatus(root, "copy.announce.copied", button);
          lastCopiedId = id;
          markLastCopied(list, id);
          showChromeNotice(
            root,
            root
              .querySelector?.(
                '[data-action-message][data-i18n="copy.announce.copied"]',
              )
              ?.textContent?.trim() ?? "",
            "ok",
            id,
          );
        } catch {
          applyActionStatus(root, "copy.announce.failed", button);
        }
        return;
      }
      if (action === "edit") {
        closeOverflowMenus(root);
        const row = button.closest("li");
        const next = await openEditSheet(root, row?.dataset?.body ?? "");
        if (next !== null) {
          const updated = await invokeFn("edit_queue_item", { id, body: next });
          await refresh({ action: "replace", item: updated });
        }
        return;
      }
      await applyQueueItemMutation(invokeFn, { id, action }, refresh);
    });
  });

  list.addEventListener("focusin", (event) => {
    if (queueFocusAtEnd(queueItemRows(list), event.target)) {
      loadMore();
    }
  });
  list.addEventListener("keydown", (event) => {
    if (
      shouldLoadNextOnKey({
        key: event.key,
        onLastCard: queueFocusAtEnd(queueItemRows(list), event.target),
        hasCursor: Boolean(state.nextCursor),
      })
    ) {
      event.preventDefault();
      loadMore();
    }
  });
  root.addEventListener?.("bronze-notice-open", (event) => {
    const id = event.detail?.id;
    if (id) {
      revealCopied(id);
    }
  });
  const sentinel = root.querySelector("[data-queue-sentinel]");
  if (sentinel && typeof IntersectionObserver === "function") {
    const scroller = list.closest?.("#quick-panel") ?? null;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          loadMore();
        }
      },
      { root: scroller, rootMargin: "240px" },
    );
    observer.observe(sentinel);
  }
  listenQueueChanged(() => {
    refresh();
  });
  listenQueueSortChanged(() => {
    refresh({ resetPage: true });
  });
  listenCaptureResult((event) => {
    applyCaptureResult(root, event?.payload ?? event);
    if (event?.payload?.terminal === "saved" || event?.terminal === "saved") {
      refresh({ action: "insert", followNew: true });
    }
  });
  root.addEventListener?.(LOCALE_APPLIED_EVENT, () => {
    refresh({ action: "replace" });
    syncSubmitLabel();
  });

  try {
    await refresh();
  } catch {
    list.replaceChildren();
  }
}

if (globalThis.document?.readyState) {
  bindQueueLive();
}
