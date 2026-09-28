import { MOTION, motionAllowed } from "./control.mjs";

export const ARRIVE_SLIDE_MS = 380;
export const ARRIVE_ROOM_MS = 320;
export const ARRIVE_SLIDE_DELAY_MS = 80;
export const ARRIVE_SCROLL_MS = 340;
export const RING_PULSE_MS = 1000;
export const ARRIVAL_MS = ARRIVE_SCROLL_MS;

function armArrivalRing(card) {
  if (!card?.classList) {
    return;
  }
  if (!card.classList.contains("is-ring-pulse")) {
    card.classList.add("is-ring-pulse");
  }
  const drop = () => {
    card.classList?.remove("is-ring-pulse");
  };
  const article =
    typeof card.querySelector === "function"
      ? card.querySelector("article")
      : null;
  if (article && typeof article.addEventListener === "function") {
    article.addEventListener(
      "animationend",
      (event) => {
        if (
          event?.animationName &&
          event.animationName !== "bronze-copy-ring"
        ) {
          return;
        }
        drop();
      },
      { once: true },
    );
  }
  const view = card.ownerDocument?.defaultView;
  if (typeof view?.setTimeout === "function") {
    view.setTimeout(drop, RING_PULSE_MS);
  }
}

const NEWEST = new Set(["newest", "created-desc", "createddesc"]);

export function normalizeQueueSort(value) {
  const token = String(value ?? "")
    .trim()
    .toLowerCase()
    .replace(/[\s_]+/g, "-")
    .replace(/-/g, "");
  if (token === "createddesc" || NEWEST.has(token) || token === "newest") {
    return "newest";
  }
  return "oldest";
}

export function readQueueSort(list) {
  const explicit =
    list?.dataset?.queueSort ??
    list?.ownerDocument?.documentElement?.dataset?.queueSort ??
    list?.ownerDocument?.defaultView?.__bronzeQueueSort;
  if (explicit == null || explicit === "") {
    return "oldest";
  }
  return normalizeQueueSort(explicit);
}

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

export function shouldLoadNextOnKey({ key, onLastCard, hasCursor } = {}) {
  const atEnd = key === "ArrowDown" || key === "End" || key === "PageDown";
  return Boolean(atEnd && onLastCard && hasCursor);
}

export function planNewItemFollow({ sort, prevIds = [], nextIds = [] } = {}) {
  const mode = typeof sort === "string" ? normalizeQueueSort(sort) : "oldest";
  const prev = new Set(prevIds);
  const added = nextIds.filter((id) => id && !prev.has(id));
  const atLoadedStart = added.length === 1 && nextIds[0] === added[0];
  if (mode === "newest" && atLoadedStart) {
    return { scrollId: added[0], cursor: "stay", prepend: false };
  }
  return { scrollId: null, cursor: "stay", prepend: false };
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

export function queueScrollParent(node) {
  if (!node || typeof node.closest !== "function") {
    return null;
  }
  return (
    node.closest("#quick-panel") ??
    node.closest("[data-slot='quick-panel']") ??
    node.closest("[data-slot=quick-panel]")
  );
}

function easeOut(t) {
  const clamped = Math.min(1, Math.max(0, t));
  return 1 - (1 - clamped) ** 3;
}

export function travelScroll(
  scroller,
  from,
  to,
  {
    motion = false,
    duration = MOTION.duration,
    frame,
    now,
    primeStart = false,
  } = {},
) {
  if (!scroller) {
    return { behavior: "auto" };
  }
  if (!motion || from === to) {
    if (!motion && from !== to) {
      scroller.scrollTop = to;
    }
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
    if (!primeStart) {
      scroller.scrollTop = to;
    }
    return { behavior: primeStart ? "auto" : "smooth" };
  }
  if (primeStart) {
    // Park the pre-insert offset before paint. Later frames ease to `to`.
    scroller.scrollTop = from;
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

export function scrollQueueCard(
  card,
  { motion = false, duration = MOTION.duration } = {},
) {
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
    return travelScroll(scroller, from, from + delta, { motion, duration });
  }
  const behavior = motion ? "smooth" : "auto";
  if (typeof card.scrollIntoView === "function") {
    card.scrollIntoView({ behavior, block: "nearest", inline: "nearest" });
  }
  return { behavior };
}

export function focusQueueCard(card) {
  if (!card || typeof card.focus !== "function") {
    return;
  }
  card.tabIndex = -1;
  try {
    card.focus({ preventScroll: true, focusVisible: true });
  } catch {
    try {
      card.focus({ preventScroll: true });
    } catch {
      card.focus();
    }
  }
}

export function findQueueCard(list, id) {
  if (!list || typeof list.querySelector !== "function") {
    return null;
  }
  if (typeof id !== "string" || !/^[A-Za-z0-9_-]{1,80}$/.test(id)) {
    return null;
  }
  return list.querySelector(`[data-item-id="${id}"]`);
}

function rowGapPx(list) {
  const view = list?.ownerDocument?.defaultView;
  const style = view?.getComputedStyle?.(list);
  const raw =
    style?.rowGap && style.rowGap !== "normal" ? style.rowGap : style?.gap;
  const parsed = parseFloat(raw);
  return Number.isFinite(parsed) ? parsed : 16;
}

function isQueueList(node) {
  return node?.id === "queue" || node?.id === "library-queue";
}

function listChildren(list) {
  if (!list) {
    return [];
  }
  const kids = list.children;
  if (Array.isArray(kids)) {
    return kids;
  }
  if (kids && typeof kids.length === "number") {
    return Array.from(kids);
  }
  return [];
}

export function insertNeighbor(card) {
  const parent = card?.parentElement;
  if (!parent || isQueueList(card)) {
    return { neighbor: null, atStart: false };
  }
  const kids = listChildren(parent);
  const index = kids.indexOf(card);
  if (index === 0 && kids.length > 1) {
    return { neighbor: kids[1], atStart: true };
  }
  if (index > 0 && index === kids.length - 1) {
    return { neighbor: kids[index - 1], atStart: false };
  }
  if (index >= 0) {
    return { neighbor: null, atStart: index === 0 };
  }
  if (parent.firstElementChild === card) {
    return {
      neighbor: card.nextElementSibling ?? kids[1] ?? null,
      atStart: true,
    };
  }
  if (parent.lastElementChild === card) {
    return {
      neighbor: card.previousElementSibling ?? null,
      atStart: false,
    };
  }
  return { neighbor: null, atStart: false };
}

function slideDelayMs(neighbor, atStart) {
  if (!neighbor) {
    return 0;
  }
  return atStart ? ARRIVE_SLIDE_DELAY_MS : ARRIVE_ROOM_MS;
}

function clearMotionInline(el) {
  if (!el?.style) {
    return;
  }
  el.style.removeProperty?.("transform");
  el.style.removeProperty?.("margin");
  el.style.removeProperty?.("margin-block-start");
  el.style.removeProperty?.("margin-block-end");
  el.style.removeProperty?.("--arrive-block");
  el.style.removeProperty?.("--arrive-slide-delay");
}

function clearInsertMotion(card, neighbor, list) {
  card?.classList?.remove?.("is-slide-in");
  card?.classList?.remove?.("is-slot-pending");
  card?.classList?.remove?.("is-slide-docked");
  neighbor?.classList?.remove?.("is-make-room");
  neighbor?.classList?.remove?.("is-make-room-end");
  clearMotionInline(card);
  clearMotionInline(neighbor);
  list?.style?.removeProperty?.("--arrive-block");
}

function insertTiming(card) {
  const { neighbor, atStart } = insertNeighbor(card);
  const delayMs = slideDelayMs(neighbor, atStart);
  return {
    neighbor,
    atStart,
    delayMs,
    roomMs: neighbor ? ARRIVE_ROOM_MS : 0,
    slideMs: delayMs + ARRIVE_SLIDE_MS,
  };
}

export function armNewCardSlide(card) {
  if (!card || isQueueList(card)) {
    return { delayMs: 0, roomMs: 0, slideMs: 0 };
  }
  if (card.dataset?.slideParked === "1") {
    return insertTiming(card);
  }
  const height = Number(card.getBoundingClientRect?.().height);
  const list = card.parentElement;
  const gap = rowGapPx(list);
  const block = Number.isFinite(height) && height > 0 ? height + gap : 0;
  const timing = insertTiming(card);
  const { neighbor, atStart, delayMs, roomMs, slideMs } = timing;
  if (list?.style?.setProperty && block > 0) {
    list.style.setProperty("--arrive-block", `${block}px`);
  }
  if (card.style?.setProperty) {
    card.style.setProperty("--arrive-slide-delay", `${delayMs}ms`);
  }
  if (card.dataset) {
    card.dataset.slideParked = "1";
  }
  if (neighbor && block > 0) {
    neighbor.classList?.add?.("is-make-room");
    if (!atStart) {
      neighbor.classList?.add?.("is-make-room-end");
      card.classList?.add?.("is-slot-pending");
    }
  }
  card.classList?.add?.("is-slide-in");
  const view = card.ownerDocument?.defaultView;
  let roomDone = !neighbor || block <= 0;
  let slideDone = false;
  const finishRoom = () => {
    if (roomDone) {
      return;
    }
    roomDone = true;
    card.classList?.remove?.("is-slot-pending");
    neighbor?.classList?.remove?.("is-make-room");
    neighbor?.classList?.remove?.("is-make-room-end");
    clearMotionInline(neighbor);
  };
  const finishSlide = () => {
    if (slideDone) {
      return;
    }
    slideDone = true;
    finishRoom();
    card.classList?.remove?.("is-slide-in");
    card.classList?.remove?.("is-slide-docked");
    clearMotionInline(card);
    list?.style?.removeProperty?.("--arrive-block");
  };
  if (typeof view?.setTimeout === "function") {
    if (!roomDone) {
      view.setTimeout(finishRoom, roomMs);
    }
    view.setTimeout(finishSlide, slideMs);
  }
  if (typeof card.addEventListener === "function") {
    const onEnd = (event) => {
      if (event?.target && event.target !== card) {
        return;
      }
      const name = event?.animationName;
      if (
        name &&
        name !== "bronze-slide-in" &&
        name !== "bronze-slide-in-rtl"
      ) {
        return;
      }
      card.removeEventListener("animationend", onEnd);
      finishSlide();
    };
    card.addEventListener("animationend", onEnd);
  }
  return { delayMs, roomMs, slideMs };
}

export function insertionBeforeId(loadedIds, pageIds, itemId) {
  const loaded = new Set(loadedIds);
  const index = pageIds.indexOf(itemId);
  if (index < 0) {
    return null;
  }
  for (let i = index + 1; i < pageIds.length; i += 1) {
    if (loaded.has(pageIds[i])) {
      return pageIds[i];
    }
  }
  return null;
}

export function presentNewQueueCard(
  card,
  scroller,
  {
    motion = false,
    followScroll = false,
    pulse = false,
    frame,
    now,
    duration = ARRIVE_SCROLL_MS,
  } = {},
) {
  if (!card?.classList || isQueueList(card)) {
    return { behavior: "auto", scrolled: false, from: null, to: null };
  }
  card.classList.remove("is-entering");
  card.classList.remove("is-arriving");
  if (!motion) {
    const { neighbor } = insertNeighbor(card);
    clearInsertMotion(card, neighbor, card.parentElement);
    card.classList.remove("is-ring-pulse");
    return { behavior: "auto", scrolled: false, from: null, to: null };
  }
  const plan = armNewCardSlide(card);
  if (!card.classList.contains("is-slide-in")) {
    card.classList.add("is-slide-in");
  }
  if (pulse) {
    const view = card.ownerDocument?.defaultView;
    if (typeof view?.setTimeout === "function") {
      if (plan.slideMs > 0) {
        view.setTimeout(() => armArrivalRing(card), plan.slideMs);
      } else {
        armArrivalRing(card);
      }
    }
  } else {
    card.classList.remove("is-ring-pulse");
  }
  if (!followScroll || !scroller) {
    return { behavior: "auto", scrolled: false, from: null, to: null };
  }
  const from = Number(scroller.scrollTop ?? 0);
  if (!Number.isFinite(from)) {
    return { behavior: "auto", scrolled: false, from: null, to: null };
  }
  const to = 0;
  const travel = travelScroll(scroller, from, to, {
    motion: true,
    duration,
    frame,
    now,
  });
  return { ...travel, from, to, scrolled: travel.behavior === "smooth" };
}

export function markLastCopied(list, id, { pulse = false } = {}) {
  if (!list || typeof list.querySelectorAll !== "function") {
    return;
  }
  for (const row of list.querySelectorAll(".queue-item")) {
    if (!row?.classList) {
      continue;
    }
    const copied = Boolean(id) && row.dataset?.itemId === id;
    row.classList.toggle("is-last-copied", copied);
    if (!copied) {
      row.classList.remove("is-ring-pulse");
    } else if (pulse && !row.classList.contains("is-ring-pulse")) {
      armArrivalRing(row);
    }
  }
}

export async function revealQueueItem({
  id,
  list,
  invoke,
  doc,
  template,
  insertMissing,
  sort,
} = {}) {
  if (!id || !list) {
    return { found: false, behavior: "auto" };
  }
  let card = findQueueCard(list, id);
  if (!card && typeof invoke === "function") {
    const mode =
      typeof sort === "string" ? normalizeQueueSort(sort) : readQueueSort(list);
    const page = await invoke("queue_query", {
      filter: "overview",
      limit: 20,
      sort: mode,
      itemId: id,
    });
    const items = page?.items ?? [];
    if (
      items.some((item) => item?.id === id) &&
      typeof insertMissing === "function"
    ) {
      insertMissing(list, items, template);
    }
    card = findQueueCard(list, id);
  }
  if (!card) {
    return { found: false, behavior: "auto" };
  }
  const motion = motionAllowed(doc ?? card.ownerDocument);
  const scrolled = scrollQueueCard(card, { motion });
  focusQueueCard(card);
  return { found: true, behavior: scrolled.behavior, id };
}
