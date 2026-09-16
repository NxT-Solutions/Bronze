import { tauriInvoke } from "./tauri-bridge.mjs";

export const QUE_007_COMPLETE = false;
export const ADR_018_STATUS = "Proposed";

export function announceCount(count) {
  return `${count} items`;
}

export async function bindLibraryLive(root = document, invokeFn = tauriInvoke) {
  const search = root.querySelector("#library-search");
  const count = root.querySelector("[data-search-count]");
  const empty = root.querySelector(".empty-state");
  const status = root.querySelector("[data-restore-preview]");
  const list = root.querySelector("#library-queue");

  async function refresh(query) {
    const includeTrash = root.location?.hash === "#trash";
    let items;
    if (query) {
      items = await invokeFn("search_library_items", { query });
    } else {
      items = await invokeFn("list_queue_items", {
        includeTrashed: includeTrash,
      });
    }
    if (count) {
      count.textContent = announceCount(items.length);
    }
    if (empty) {
      empty.hidden = items.length > 0;
    }
    if (list) {
      list.replaceChildren();
      for (const item of items) {
        const li = document.createElement("li");
        const article = document.createElement("article");
        article.lang = item.contentLanguage || "und";
        article.dir = "auto";
        const body = document.createElement("p");
        body.textContent = item.body;
        article.append(body);
        li.append(article);
        list.append(li);
      }
    }
    return items;
  }

  search?.addEventListener("input", () => {
    refresh(search.value).catch(() => {
      if (count) {
        count.textContent = announceCount(0);
      }
    });
  });

  root
    .querySelector("[data-i18n='library.backup.now']")
    ?.addEventListener("click", async () => {
      const path = await invokeFn("backup_library_now", {
        requestedPath: null,
      });
      if (status) {
        status.textContent = path;
      }
    });
  root
    .querySelector("[data-i18n='library.export']")
    ?.addEventListener("click", async () => {
      const path = await invokeFn("export_library_archive", {
        requestedPath: null,
      });
      if (status) {
        status.textContent = path;
      }
    });
  root
    .querySelector("[data-i18n='library.import']")
    ?.addEventListener("click", async () => {
      const imported = await invokeFn("import_library_archive", {
        snapshot: null,
        requestedPath: null,
      });
      if (status) {
        status.textContent = announceCount(imported);
      }
      await refresh(search?.value ?? "");
    });

  try {
    await refresh("");
  } catch {
    if (count) {
      count.textContent = announceCount(0);
    }
  }
  return list;
}

if (globalThis.document?.readyState) {
  bindLibraryLive();
}
