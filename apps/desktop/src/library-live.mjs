import {
  catalogMessage,
  formatQueueCount,
  LOCALE_APPLIED_EVENT,
} from "./apply-locale.mjs";
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
  return formatQueueCount(count, catalogMessage("queue.count"));
}

export function libraryEmptyKey(section) {
  if (section === "trash") {
    return "library.state.emptyTrash";
  }
  if (section === "search") {
    return "library.state.emptySearch";
  }
  return "library.state.empty";
}

export function applyLibraryEmptyCopy(root, section) {
  const title = root.querySelector?.("#library-empty-title");
  const key = libraryEmptyKey(section);
  const source = root.querySelector?.(
    `[data-empty-message][data-i18n="${key}"]`,
  );
  if (title && source) {
    title.textContent = source.textContent.trim();
  }
  return key;
}

export function librarySection(hash) {
  if (hash === "#trash") {
    return "trash";
  }
  if (hash === "#search") {
    return "search";
  }
  return "archive";
}

export function libraryListMode(hash, query) {
  if (String(query ?? "").trim()) {
    return "search";
  }
  return librarySection(hash) === "trash" ? "trash" : "archive";
}

export function syncLibraryNav(root, hash) {
  const current = librarySection(hash ?? root.location?.hash);
  const href = current === "trash" ? "#trash" : "#archive";
  const links = root.querySelectorAll?.(".segment a") ?? [];
  for (const link of links) {
    if (link.getAttribute("href") === href) {
      link.setAttribute("aria-current", "page");
    } else {
      link.removeAttribute("aria-current");
    }
  }
  return current;
}

function bindSearchClear(root) {
  const wrap = root.querySelector?.(".chrome-search");
  const input = wrap?.querySelector?.("input[type='search']");
  const clear = wrap?.querySelector?.("[data-search-clear]");
  if (!wrap || !input || !clear) {
    return;
  }
  const sync = () => {
    const filled = String(input.value ?? "").trim().length > 0;
    clear.hidden = !filled;
    wrap.classList.toggle("is-filled", filled);
  };
  input.addEventListener("input", sync);
  clear.addEventListener("click", () => {
    input.value = "";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.focus();
  });
  sync();
}

export async function bindLibraryLive(root = document, invokeFn = tauriInvoke) {
  const search = root.querySelector("#library-search");
  const count = root.querySelector("[data-search-count]");
  const empty = root.querySelector(".empty-state");
  const status = root.querySelector("[data-restore-preview]");
  const list = root.querySelector("#library-queue");
  const template = root.querySelector("#library-item-template");

  async function refresh(query) {
    const section = syncLibraryNav(root);
    const mode = libraryListMode(root.location?.hash ?? "", query);
    const includeTrash = section === "trash";
    let items;
    if (mode === "search") {
      items = await invokeFn("search_library_items", {
        query: String(query ?? "").trim(),
      });
    } else {
      items = await invokeFn("list_queue_items", {
        includeTrashed: includeTrash,
      });
    }
    if (count) {
      count.textContent = announceCount(items.length);
    }
    applyLibraryEmptyCopy(root, mode);
    if (empty) {
      empty.hidden = items.length > 0;
    }
    if (list && template) {
      const labels = readExpandLabels(template.content);
      list.replaceChildren();
      for (const [index, item] of items.entries()) {
        const node = template.content.firstElementChild.cloneNode(true);
        node.classList.add("is-entering");
        node.style.setProperty("--enter-delay", `${Math.min(index, 8) * 24}ms`);
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

  bindSearchClear(root);

  search?.addEventListener("input", () => {
    refresh(search.value).catch(() => {
      if (count) {
        count.textContent = announceCount(0);
      }
    });
  });

  root.defaultView?.addEventListener("hashchange", () => {
    const section = syncLibraryNav(root);
    if (section === "search") {
      search?.focus();
    }
    refresh(search?.value ?? "").catch(() => {
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

  root.addEventListener?.(LOCALE_APPLIED_EVENT, () => {
    refresh(search?.value ?? "").catch(() => {
      if (count) {
        count.textContent = announceCount(0);
      }
    });
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
