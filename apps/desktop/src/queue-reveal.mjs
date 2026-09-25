import { MOTION, motionAllowed } from "./control.mjs";

export function isNearLoadedStart(scroller, threshold = 24) {
  if (!scroller) {
    return true;
  }
  const top = Number(scroller.scrollTop ?? 0);
  if (!Number.isFinite(top)) {
    return true;
  }
  return top <= threshold;
}

export function planNewItemFollow({
  sort,
  nearStart = false,
  prevIds = [],
  nextIds = [],
} = {}) {
  const newest = sort === "newest";
  const prev = new Set(prevIds);
  const added = nextIds.filter((id) => id && !prev.has(id));
  const atLoadedEdge = added.length === 1 && nextIds[0] === added[0];
  if (newest && nearStart && atLoadedEdge) {
    return { scrollId: added[0] };
  }
  return { scrollId: null };
}

export function shouldLoadNextOnKey({ key, onLastCard, hasCursor }) {
  const atEnd = key === "ArrowDown" || key === "End" || key === "PageDown";
  return Boolean(atEnd && onLastCard && hasCursor);
}

export function findQueueCard(list, id) {
  if (!list || typeof list.querySelectorAll !== "function" || !id) {
    return null;
  }
  for (const row of list.querySelectorAll(".queue-item")) {
    if (row?.dataset?.itemId === id) {
      return row;
    }
  }
  return null;
}

export function markLastCopied(list, id) {
  if (!list || typeof list.querySelectorAll !== "function") {
    return;
  }
  for (const row of list.querySelectorAll(".queue-item")) {
    const on = Boolean(id) && row.dataset?.itemId === id;
    row.classList?.toggle?.("is-last-copied", on);
    const label = row.querySelector?.("[data-slot=last-copied]");
    if (label) {
      label.hidden = !on;
    }
  }
}

export function scrollDelta(cardRect, viewRect) {
  if (!cardRect || !viewRect) {
    return 0;
  }
  if (cardRect.top >= viewRect.top && cardRect.bottom <= viewRect.bottom) {
    return 0;
  }
  if (cardRect.top < viewRect.top) {
    return cardRect.top - viewRect.top;
  }
  return cardRect.bottom - viewRect.bottom;
}

function easeOut(t) {
  const clamped = Math.min(1, Math.max(0, t));
  return 1 - (1 - clamped) ** 3;
}

export function travelScroll(
  scroller,
  from,
  to,
  { motion = false, duration = MOTION.duration, frame, now } = {},
) {
  if (!scroller) {
    return { behavior: "auto" };
  }
  if (!motion || from === to) {
    scroller.scrollTop = to;
    return { behavior: "auto" };
  }
  const schedule =
    frame ?? scroller.ownerDocument?.defaultView?.requestAnimationFrame;
  const clock =
    now ??
    scroller.ownerDocument?.defaultView?.performance?.now?.bind(
      scroller.ownerDocument.defaultView.performance,
    );
  if (typeof schedule !== "function" || typeof clock !== "function") {
    scroller.scrollTop = to;
    return { behavior: "auto" };
  }
  const start = clock();
  const step = (time) => {
    const span = duration <= 0 ? 1 : Math.min(1, (time - start) / duration);
    scroller.scrollTop = from + (to - from) * easeOut(span);
    if (span < 1) {
      schedule(step);
    }
  };
  schedule(step);
  return { behavior: "smooth" };
}

export function queueScrollParent(node) {
  if (!node || typeof node.closest !== "function") {
    return null;
  }
  return (
    node.closest("#quick-panel") ?? node.closest("[data-slot=quick-panel]")
  );
}

export function scrollQueueCard(card, { motion = false } = {}) {
  if (!card) {
    return { behavior: "auto" };
  }
  const scroller = queueScrollParent(card);
  if (
    scroller &&
    typeof card.getBoundingClientRect === "function" &&
    typeof scroller.getBoundingClientRect === "function"
  ) {
    const delta = scrollDelta(
      card.getBoundingClientRect(),
      scroller.getBoundingClientRect(),
    );
    const from = Number(scroller.scrollTop ?? 0);
    return travelScroll(scroller, from, from + delta, { motion });
  }
  if (typeof card.scrollIntoView === "function") {
    card.scrollIntoView({
      behavior: motion ? "smooth" : "auto",
      block: "nearest",
      inline: "nearest",
    });
  }
  return { behavior: motion ? "smooth" : "auto" };
}

export function focusQueueCard(card) {
  if (!card || typeof card.focus !== "function") {
    return;
  }
  card.tabIndex = -1;
  try {
    card.focus({ preventScroll: true });
  } catch {
    card.focus();
  }
}

export async function openCopiedNotice({
  id,
  loaded = [],
  invoke,
  sort = "newest",
  motion = false,
  scroller,
  card,
} = {}) {
  let items = loaded.slice();
  let fetched = false;
  let nextCursor = null;
  if (!items.some((item) => item.id === id)) {
    const page = await invoke("queue_page_for_item", { id, sort });
    items = Array.isArray(page?.items) ? page.items : [];
    nextCursor = page?.nextCursor ?? null;
    fetched = true;
  }
  if (scroller && card) {
    const delta = scrollDelta(
      card.getBoundingClientRect?.(),
      scroller.getBoundingClientRect?.(),
    );
    const from = Number(scroller.scrollTop ?? 0);
    travelScroll(scroller, from, from + delta, { motion });
  }
  return {
    items,
    nextCursor,
    fetched,
    found: items.some((item) => item.id === id),
    behavior: motion ? "smooth" : "auto",
  };
}

export function motionForReveal(doc) {
  return motionAllowed(doc);
}
