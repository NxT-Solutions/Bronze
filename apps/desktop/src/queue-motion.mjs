import { animateElement, MOTION, motionAllowed } from "./control.mjs";
import {
  applySourceRow,
  fillItemChrome,
  formatCaptureSource,
  readExpandLabels,
  syncExpandVisibility,
} from "./item-view.mjs";

const EXIT_ACTIONS = new Set(["complete", "skip", "trash"]);
const MOVE_ACTIONS = new Set(["moveUp", "moveDown"]);

export function queueMotionKind({ prevIds = [], nextIds = [], action } = {}) {
  if (action === "replace") {
    return "replace";
  }
  if (EXIT_ACTIONS.has(action)) {
    return "exit";
  }
  if (MOVE_ACTIONS.has(action)) {
    return "move";
  }
  if (action === "insert") {
    return "enter";
  }
  const prev = new Set(prevIds);
  const next = new Set(nextIds);
  const removed = prevIds.filter((id) => !next.has(id));
  const added = nextIds.filter((id) => !prev.has(id));
  if (removed.length > 0 && added.length === 0) {
    return "exit";
  }
  if (added.length > 0 && removed.length === 0) {
    return "enter";
  }
  if (
    removed.length === 0 &&
    added.length === 0 &&
    prevIds.length === nextIds.length &&
    prevIds.some((id, index) => id !== nextIds[index])
  ) {
    return "move";
  }
  if (prevIds.length === 0) {
    return "enter";
  }
  return "replace";
}

export function exitMotionClass(action) {
  if (action === "complete") {
    return "is-leaving-complete";
  }
  if (action === "trash") {
    return "is-leaving-trash";
  }
  return "is-leaving-skip";
}

export function shouldAnimateQueue(doc, kind) {
  if (!motionAllowed(doc)) {
    return false;
  }
  return kind === "exit" || kind === "move" || kind === "enter";
}

export function waitForQueueMotion(el, durationMs = MOTION.duration) {
  const view = el?.ownerDocument?.defaultView;
  if (!el || typeof el.addEventListener !== "function") {
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    let settled = false;
    const finish = () => {
      if (settled) {
        return;
      }
      settled = true;
      el.removeEventListener("transitionend", onEnd);
      resolve();
    };
    function onEnd(event) {
      if (event?.target === el) {
        finish();
      }
    }
    el.addEventListener("transitionend", onEnd);
    if (typeof view?.setTimeout === "function") {
      view.setTimeout(finish, Number(durationMs) + 32);
    } else {
      finish();
    }
  });
}

export async function applyQueueExitMotion(node, action) {
  if (!node) {
    return { animated: false, className: null };
  }
  if (!motionAllowed(node.ownerDocument)) {
    return { animated: false, className: null };
  }
  const className = exitMotionClass(action);
  if (typeof node.getBoundingClientRect === "function" && node.style) {
    const height = node.getBoundingClientRect().height;
    if (Number.isFinite(height) && height > 0) {
      node.style.height = `${height}px`;
      node.style.overflow = "hidden";
      node.getBoundingClientRect();
      node.style.height = "0px";
    }
  }
  node.classList?.add?.(className);
  try {
    await waitForQueueMotion(node, MOTION.duration);
  } catch {}
  return { animated: true, className };
}

export async function applyQueueItemMutation(
  invokeFn,
  { id, action },
  refresh,
) {
  await invokeFn("apply_queue_item_action", { id, action });
  try {
    await refresh({ action, id });
  } catch {
    try {
      await refresh({ action: "replace", id });
    } catch {}
  }
}

function fillQueueNode(node, item, labels) {
  node.dataset.itemId = item.id;
  node.dataset.body = typeof item.body === "string" ? item.body : "";
  const article = node.querySelector("article");
  fillItemChrome(article, item, labels);
  const source = node.querySelector("[data-slot=source]");
  const labelNode = source?.querySelector("[data-slot=source-label]") ?? source;
  const label = formatCaptureSource(labelNode?.textContent, item.sourceAppName);
  applySourceRow(article, label, item.sourceAppIcon);
  node.querySelectorAll("[data-queue-action]").forEach((button) => {
    button.dataset.itemId = item.id;
  });
  syncExpandVisibility(article);
}

export function createQueueRenderer({ queueItemRows, syncMoveAvailability }) {
  function listItemIds(list) {
    return queueItemRows(list)
      .map((el) => el.dataset?.itemId)
      .filter(Boolean);
  }

  function paintQueueItems(list, items, template, { enteringIds } = {}) {
    const labels = readExpandLabels(template.content);
    const enter = enteringIds ?? new Set();
    list.replaceChildren();
    for (const [index, item] of items.entries()) {
      const node = template.content.firstElementChild.cloneNode(true);
      if (enter.has(item.id)) {
        node.classList.add("is-entering");
        node.style.setProperty("--enter-delay", `${Math.min(index, 8) * 24}ms`);
      }
      fillQueueNode(node, item, labels);
      list.append(node);
    }
    syncMoveAvailability(list);
  }

  function reorderQueueNodes(list, items, template) {
    const labels = readExpandLabels(template.content);
    const byId = new Map();
    for (const el of queueItemRows(list)) {
      if (el.dataset?.itemId) {
        byId.set(el.dataset.itemId, el);
      }
    }
    for (const item of items) {
      let node = byId.get(item.id);
      if (!node) {
        node = template.content.firstElementChild.cloneNode(true);
        fillQueueNode(node, item, labels);
      }
      list.append(node);
    }
    for (const [id, node] of byId) {
      if (!items.some((item) => item.id === id)) {
        node.remove();
      }
    }
    syncMoveAvailability(list);
  }

  async function applyQueueMoveMotion(list, items, template) {
    const first = new Map();
    for (const el of queueItemRows(list)) {
      const id = el.dataset?.itemId;
      if (!id || typeof el.getBoundingClientRect !== "function") {
        continue;
      }
      first.set(id, el.getBoundingClientRect());
    }
    reorderQueueNodes(list, items, template);
    if (!motionAllowed(list.ownerDocument)) {
      return;
    }
    const plays = [];
    for (const el of queueItemRows(list)) {
      const prev = first.get(el.dataset?.itemId);
      if (!prev || typeof el.getBoundingClientRect !== "function") {
        continue;
      }
      const next = el.getBoundingClientRect();
      const dy = prev.top - next.top;
      if (Math.abs(dy) < 0.5) {
        continue;
      }
      const anim = animateElement(
        el,
        [{ transform: `translateY(${dy}px)` }, { transform: "none" }],
        { duration: MOTION.duration, easing: MOTION.easing },
      );
      if (anim?.finished && typeof anim.finished.then === "function") {
        plays.push(anim.finished.catch(() => {}));
      }
    }
    await Promise.all(plays);
  }

  async function renderQueueItems(list, items, template, options = {}) {
    const prevIds = listItemIds(list);
    const nextIds = items.map((item) => item.id);
    const kind = queueMotionKind({
      prevIds,
      nextIds,
      action: options.action,
    });
    const allow = shouldAnimateQueue(list?.ownerDocument, kind);

    if (kind === "exit" && allow) {
      const leaving = new Set(prevIds.filter((id) => !nextIds.includes(id)));
      const leavingNodes = queueItemRows(list).filter((el) =>
        leaving.has(el.dataset?.itemId),
      );
      try {
        await Promise.all(
          leavingNodes.map((node) =>
            applyQueueExitMotion(node, options.action),
          ),
        );
      } catch {}
      paintQueueItems(list, items, template);
      return;
    }

    if (kind === "move") {
      if (allow) {
        try {
          await applyQueueMoveMotion(list, items, template);
        } catch {
          reorderQueueNodes(list, items, template);
        }
        return;
      }
      reorderQueueNodes(list, items, template);
      return;
    }

    const enteringIds =
      kind === "enter"
        ? new Set(
            nextIds.filter(
              (id) => prevIds.length === 0 || !prevIds.includes(id),
            ),
          )
        : new Set();
    paintQueueItems(list, items, template, { enteringIds });
  }

  return { paintQueueItems, renderQueueItems };
}
