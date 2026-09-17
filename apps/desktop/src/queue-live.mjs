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
import { tauriInvoke } from "./tauri-bridge.mjs";

export { formatCaptureSource };

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
  const textarea = root.querySelector("#composer-body");
  const error = root.querySelector("#composer-error");
  const empty = root.querySelector("#queue-empty");
  const template = root.querySelector("#queue-item-template");
  const profile = root.querySelector("#output-profile");
  if (!list || !form || !textarea || !template) {
    return;
  }

  bindOverflowDismiss(root);

  async function refresh() {
    const items = await invokeFn("list_overview_items");
    renderQueueItems(list, items, template);
    if (empty) {
      empty.hidden = items.length > 0;
    }
  }

  async function addFromComposer() {
    try {
      await invokeFn("add_composer_item", { body: textarea.value });
      textarea.value = "";
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
  textarea.addEventListener("keydown", (event) => {
    if (composerShouldSubmit(event)) {
      event.preventDefault();
      const submit = form.querySelector("[type=submit]");
      runBusy(submit, addFromComposer);
    }
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

  try {
    await refresh();
  } catch {
    list.replaceChildren();
  }
}

if (globalThis.document?.readyState) {
  bindQueueLive();
}
