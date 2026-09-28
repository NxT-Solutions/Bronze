import { tauriEmit, tauriListen } from "./tauri-bridge.mjs";

export const QUEUE_SORT_EVENT = "queue-sort-changed";

export function parseQueueSort(raw) {
  return raw === "oldest" ? "oldest" : "newest";
}

export function queueSortFromEvent(payload) {
  if (typeof payload === "string") {
    return parseQueueSort(payload);
  }
  if (payload && typeof payload.sort === "string") {
    return parseQueueSort(payload.sort);
  }
  return null;
}

// A v2 cursor belongs to one sort. A different sort starts again at the first page.
export function queueListArgs(sort, cursor) {
  const next = parseQueueSort(sort);
  const token = typeof cursor === "string" ? cursor : "";
  const sameSort = token.startsWith(`v2.${next}.`);
  return {
    sort: next,
    cursor: sameSort ? token : null,
  };
}

export function queueSortChange(before, after) {
  const prev = parseQueueSort(before);
  const next = parseQueueSort(after);
  if (prev === next) {
    return null;
  }
  return { sort: next };
}

export function nextQueueSort(raw) {
  return parseQueueSort(raw) === "newest" ? "oldest" : "newest";
}

export function queueSortActionKey(raw) {
  return parseQueueSort(raw) === "newest"
    ? "queue.sort.showOldest"
    : "queue.sort.showNewest";
}

export function patchSettingsQueueSort(settings, sort) {
  const queueSort = parseQueueSort(sort);
  return {
    ...settings,
    copy: { ...(settings?.copy ?? {}), queueSort },
  };
}

export function applyQueueSortControl(button, sort, messages = {}) {
  const current = parseQueueSort(sort);
  if (!button) {
    return current;
  }
  const key = queueSortActionKey(current);
  const label = messages[key];
  if (button.dataset) {
    button.dataset.queueSort = current;
  }
  button.setAttribute?.("data-i18n-aria-label", key);
  button.setAttribute?.("data-i18n-title", key);
  if (typeof label === "string" && label) {
    button.setAttribute?.("aria-label", label);
    button.setAttribute?.("title", label);
  }
  return current;
}

export async function persistQueueSort(invokeFn, settings, sort) {
  const queueSort = parseQueueSort(sort);
  const next = patchSettingsQueueSort(settings, queueSort);
  const saved = await invokeFn("save_settings_v1", { settings: next });
  await emitQueueSortChanged({ sort: queueSort });
  return saved ?? next;
}

export function emitQueueSortChanged(payload) {
  try {
    const channel = new BroadcastChannel(QUEUE_SORT_EVENT);
    channel.postMessage(payload);
    channel.close();
  } catch {
    // BroadcastChannel is absent in some test runtimes.
  }
  return tauriEmit(QUEUE_SORT_EVENT, payload);
}

export function listenQueueSortChanged(handler) {
  let channel;
  try {
    channel = new BroadcastChannel(QUEUE_SORT_EVENT);
    channel.onmessage = (event) => {
      handler(event.data ?? {});
    };
  } catch {
    channel = null;
  }
  const listened = tauriListen(QUEUE_SORT_EVENT, (event) => {
    handler(event?.payload ?? {});
  });
  return () => {
    channel?.close?.();
    Promise.resolve(listened).then((unlisten) => {
      if (typeof unlisten === "function") {
        unlisten();
      }
    });
  };
}
