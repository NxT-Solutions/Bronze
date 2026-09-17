import { tauriInvoke } from "./tauri-bridge.mjs";

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

export function formatCaptureSource(template, appName) {
  if (typeof appName !== "string") {
    return null;
  }
  const name = appName.trim();
  if (
    name.length === 0 ||
    typeof template !== "string" ||
    !template.includes("{appName}")
  ) {
    return null;
  }
  return template.replaceAll("{appName}", name);
}

export function renderQueueItems(list, items, template) {
  list.replaceChildren();
  for (const item of items) {
    const node = template.content.firstElementChild.cloneNode(true);
    node.dataset.itemId = item.id;
    const article = node.querySelector("article");
    article.lang = item.contentLanguage || "und";
    article.dir = "auto";
    node.querySelector("[data-slot=body]").textContent = item.body;
    const source = node.querySelector("[data-slot=source]");
    if (source) {
      const label = formatCaptureSource(source.textContent, item.sourceAppName);
      if (label) {
        source.textContent = label;
        source.hidden = false;
      } else {
        source.hidden = true;
      }
    }
    node.querySelectorAll("[data-queue-action]").forEach((button) => {
      button.dataset.itemId = item.id;
    });
    list.append(node);
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
      addFromComposer();
    }
  });
  textarea.addEventListener("keydown", (event) => {
    if (composerShouldSubmit(event)) {
      event.preventDefault();
      addFromComposer();
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
    if (action === "copy") {
      await invokeFn("copy_queue_items", {
        itemIds: [id],
        profile: profile?.value ?? "plain",
      });
      return;
    }
    if (action === "edit") {
      const next = root.defaultView?.prompt?.("", "") ?? "";
      if (next) {
        await invokeFn("edit_queue_item", { id, body: next });
        await refresh();
      }
      return;
    }
    await invokeFn("apply_queue_item_action", { id, action });
    await refresh();
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
