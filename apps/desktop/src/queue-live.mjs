import { showChromeWindow, tauriInvoke } from "./tauri-bridge.mjs";

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

export function renderQueueItems(list, items) {
  list.replaceChildren();
  for (const item of items) {
    const li = document.createElement("li");
    li.dataset.itemId = item.id;
    const article = document.createElement("article");
    article.lang = item.contentLanguage || "und";
    article.dir = "auto";
    const body = document.createElement("p");
    body.textContent = item.body;
    const menu = document.createElement("menu");
    for (const [action, label] of [
      ["moveUp", "Move up"],
      ["moveDown", "Move down"],
      ["complete", "Complete"],
      ["skip", "Skip"],
      ["trash", "Trash"],
      ["edit", "Edit"],
    ]) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "btn-toolbar";
      button.dataset.queueAction = action;
      button.dataset.itemId = item.id;
      button.textContent = label;
      menu.append(button);
    }
    article.append(body, menu);
    li.append(article);
    list.append(li);
  }
}

export async function bindQueueLive(root = document, invokeFn = tauriInvoke) {
  const list = root.querySelector("#queue");
  const form = root.querySelector("#composer");
  const textarea = root.querySelector("#composer-body");
  const error = root.querySelector("#composer-error");
  const preview = root.querySelector("#copy-toolbar pre");
  const profile = root.querySelector("#output-profile");
  if (!list || !form || !textarea) {
    return;
  }

  async function refresh() {
    const items = await invokeFn("list_queue_items", { includeTrashed: false });
    renderQueueItems(list, items);
    if (preview && items[0]) {
      preview.textContent = items[0].body;
    }
    if (preview && items.length === 0) {
      preview.textContent = "";
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

  const copyButton = root.querySelector("#copy-toolbar .btn-primary");
  copyButton?.addEventListener("click", async () => {
    const text = await invokeFn("copy_queue_items", {
      itemIds: [],
      profile: profile?.value ?? "plain",
    });
    if (preview) {
      preview.textContent = text;
    }
  });

  root.querySelectorAll("[data-open-window]").forEach((button) => {
    button.addEventListener("click", () => {
      const kind = button.getAttribute("data-open-window");
      if (kind) {
        showChromeWindow(kind, invokeFn);
      }
    });
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
