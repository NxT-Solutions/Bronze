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
import {
  applySourceRow,
  fillItemChrome,
  formatCaptureSource,
  readExpandLabels,
  syncExpandVisibility,
} from "./item-view.mjs";
import { serializeComposerDom } from "./markdown-body.mjs";
import { tauriInvoke } from "./tauri-bridge.mjs";

export { formatCaptureSource, serializeComposerDom };

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

export function renderQueueItems(list, items, template) {
  const labels = readExpandLabels(template.content);
  list.replaceChildren();
  for (const [index, item] of items.entries()) {
    const node = template.content.firstElementChild.cloneNode(true);
    node.classList.add("is-entering");
    node.style.setProperty("--enter-delay", `${Math.min(index, 8) * 24}ms`);
    node.dataset.itemId = item.id;
    node.dataset.body = typeof item.body === "string" ? item.body : "";
    const article = node.querySelector("article");
    fillItemChrome(article, item, labels);
    const source = node.querySelector("[data-slot=source]");
    const labelNode =
      source?.querySelector("[data-slot=source-label]") ?? source;
    const label = formatCaptureSource(
      labelNode?.textContent,
      item.sourceAppName,
    );
    applySourceRow(article, label, item.sourceAppIcon);
    node.querySelectorAll("[data-queue-action]").forEach((button) => {
      button.dataset.itemId = item.id;
    });
    list.append(node);
    syncExpandVisibility(article);
  }
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
  syncComposerEmpty(editor);
  const submit = form.querySelector("[type=submit]");
  function syncSubmitLabel() {
    applyComposerSubmitLabel(submit, serializeComposerDom(editor));
  }
  syncSubmitLabel();

  async function refresh() {
    const items = await invokeFn("list_overview_items");
    renderQueueItems(list, items, template);
    if (empty) {
      empty.hidden = items.length > 0;
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
      await refresh();
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
    if (!(button instanceof HTMLButtonElement)) {
      return;
    }
    const id = button.dataset.itemId;
    const action = button.dataset.queueAction;
    if (!id || !action) {
      return;
    }
    await runBusy(button, async () => {
      if (action === "copy") {
        try {
          await invokeFn("copy_queue_items", {
            itemIds: [id],
            profile: profile?.value ?? "plain",
          });
          applyActionStatus(root, "copy.announce.copied", button);
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
          await invokeFn("edit_queue_item", { id, body: next });
          await refresh();
        }
        return;
      }
      await invokeFn("apply_queue_item_action", { id, action });
      await refresh();
    });
  });

  listenQueueChanged(() => {
    refresh();
  });
  listenCaptureResult((event) => {
    applyCaptureResult(root, event?.payload ?? event);
    if (event?.payload?.terminal === "saved" || event?.terminal === "saved") {
      refresh();
    }
  });
  root.addEventListener?.(LOCALE_APPLIED_EVENT, () => {
    refresh();
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
