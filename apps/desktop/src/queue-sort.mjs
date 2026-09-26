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
