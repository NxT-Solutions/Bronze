import { runBusy } from "./control.mjs";
import {
  applySourceRow,
  fillItemChrome,
  formatCaptureSource,
  readExpandLabels,
  syncExpandVisibility,
} from "./item-view.mjs";
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
  const template = root.querySelector("#library-item-template");

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
    if (list && template) {
      const labels = readExpandLabels(template.content);
      list.replaceChildren();
      for (const item of items) {
        const node = template.content.firstElementChild.cloneNode(true);
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
        list.append(node);
        syncExpandVisibility(article);
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

  function bindBusyClick(selector, work) {
    const button = root.querySelector(selector);
    button?.addEventListener("click", () => {
      runBusy(button, work);
    });
  }

  bindBusyClick("[data-i18n='library.backup.now']", async () => {
    const path = await invokeFn("backup_library_now", {
      requestedPath: null,
    });
    if (status) {
      status.textContent = path;
    }
  });
  bindBusyClick("[data-i18n='library.export']", async () => {
    const path = await invokeFn("export_library_archive", {
      requestedPath: null,
    });
    if (status) {
      status.textContent = path;
    }
  });
  bindBusyClick("[data-i18n='library.import']", async () => {
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
