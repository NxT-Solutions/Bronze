import { LOCALE_APPLIED_EVENT } from "./apply-locale.mjs";
import {
  applyActionStatus,
  bindOverflowDismiss,
  closeOverflowMenus,
  openEditSheet,
  runBusy,
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

export function wrapComposerSelection(root, tagName) {
  const doc = root?.ownerDocument;
  const sel = doc?.getSelection?.();
  if (!root || !doc?.createElement || !sel || sel.rangeCount === 0) {
    return;
  }
  if (!root.contains(sel.anchorNode)) {
    return;
  }
  const existing = formatAncestor(sel.anchorNode, tagName, root);
  if (existing) {
    unwrapElement(existing);
    return;
  }
  if (sel.isCollapsed) {
    return;
  }
  const range = sel.getRangeAt(0);
  const el = doc.createElement(tagName);
  el.appendChild(range.extractContents());
  range.insertNode(el);
  sel.removeAllRanges();
  const next = doc.createRange();
  next.selectNodeContents(el);
  sel.addRange(next);
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
  for (const button of root.querySelectorAll("[data-composer-format]")) {
    const tag = button.getAttribute("data-composer-format");
    const on = Boolean(anchor && formatAncestor(anchor, tag, editor));
    button.setAttribute("aria-pressed", on ? "true" : "false");
  }
}

export const QUE_007_COMPLETE = false;

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
  return Boolean(event.metaKey || event.ctrlKey);
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
  syncComposerEmpty(editor);

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
      if (error) {
        error.hidden = true;
      }
      await refresh();
    } catch {
      if (error) {
        error.hidden = false;
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
    if (tag !== "strong" && tag !== "em") {
      return;
    }
    wrapComposerSelection(editor, tag);
    syncComposerEmpty(editor);
    syncFormatPressed(root, editor);
  });
  editor.addEventListener("keydown", (event) => {
    if (composerShouldSubmit(event)) {
      event.preventDefault();
      const submit = form.querySelector("[type=submit]");
      runBusy(submit, addFromComposer);
      return;
    }
    if (event.isComposing || !(event.metaKey || event.ctrlKey)) {
      return;
    }
    const key = typeof event.key === "string" ? event.key.toLowerCase() : "";
    if (key === "b") {
      event.preventDefault();
      wrapComposerSelection(editor, "strong");
      syncFormatPressed(root, editor);
    } else if (key === "i") {
      event.preventDefault();
      wrapComposerSelection(editor, "em");
      syncFormatPressed(root, editor);
    }
  });
  editor.addEventListener("input", () => {
    syncComposerEmpty(editor);
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
