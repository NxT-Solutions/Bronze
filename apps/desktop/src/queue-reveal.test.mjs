import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  ARRIVAL_MS,
  insertionBeforeId,
  isNearLoadedStart,
  markLastCopied,
  normalizeQueueSort,
  planNewItemFollow,
  playQueueArrival,
  revealQueueItem,
  scrollDelta,
  shouldLoadNextOnKey,
  travelScroll,
} from "./queue-reveal.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const live = readFileSync(join(root, "queue-live.mjs"), "utf8");
const reveal = readFileSync(join(root, "queue-reveal.mjs"), "utf8");
const html = readFileSync(join(root, "index.html"), "utf8");

test("keyboard at the loaded end requests the next cursor page", () => {
  assert.equal(
    shouldLoadNextOnKey({
      key: "ArrowDown",
      onLastCard: true,
      hasCursor: true,
    }),
    true,
  );
  assert.equal(
    shouldLoadNextOnKey({ key: "End", onLastCard: true, hasCursor: true }),
    true,
  );
  assert.equal(
    shouldLoadNextOnKey({
      key: "PageDown",
      onLastCard: true,
      hasCursor: true,
    }),
    true,
  );
  assert.equal(
    shouldLoadNextOnKey({
      key: "ArrowDown",
      onLastCard: true,
      hasCursor: false,
    }),
    false,
  );
  assert.equal(
    shouldLoadNextOnKey({
      key: "ArrowDown",
      onLastCard: false,
      hasCursor: true,
    }),
    false,
  );
  assert.equal(
    shouldLoadNextOnKey({
      key: "ArrowUp",
      onLastCard: true,
      hasCursor: true,
    }),
    false,
  );
  assert.match(live, /shouldLoadNextOnKey/);
});

test("auto-scroll only when newest-first and already at the top", () => {
  const prev = ["a", "b"];
  const atTop = planNewItemFollow({
    sort: "newest",
    nearStart: true,
    prevIds: prev,
    nextIds: ["n", "a", "b"],
  });
  assert.deepEqual(atTop, { scrollId: "n", cursor: "stay", prepend: false });
  assert.equal(normalizeQueueSort("created-desc"), "newest");
  assert.equal(isNearLoadedStart({ scrollTop: 0 }), true);
  assert.equal(isNearLoadedStart({ scrollTop: 80 }), false);

  const scrolledAway = planNewItemFollow({
    sort: "newest",
    nearStart: false,
    prevIds: prev,
    nextIds: ["n", "a", "b"],
  });
  assert.equal(scrolledAway.scrollId, null);
  assert.equal(scrolledAway.cursor, "stay");
  assert.equal(scrolledAway.prepend, false);

  const unloaded = planNewItemFollow({
    sort: "newest",
    nearStart: true,
    prevIds: prev,
    nextIds: prev,
  });
  assert.equal(unloaded.scrollId, null);
  assert.equal(unloaded.prepend, false);
});

test("oldest-first insert does not jump the cursor page", () => {
  const plan = planNewItemFollow({
    sort: "oldest",
    nearStart: true,
    prevIds: ["a", "b"],
    nextIds: ["a", "b", "n"],
  });
  assert.equal(plan.scrollId, null);
  assert.equal(plan.cursor, "stay");
  assert.equal(plan.prepend, false);
  assert.equal(normalizeQueueSort("rank"), "oldest");
  assert.equal(normalizeQueueSort(""), "oldest");
  const endNeighbor = insertionBeforeId(["a", "b"], ["a", "b", "n"], "n");
  assert.equal(endNeighbor, null);
  assert.equal(insertionBeforeId(["b"], ["n", "b"], "n"), "b");
  const trailing = arrivalList("n", 1);
  const stayed = playQueueArrival(trailing.list, "n", { motion: true });
  assert.equal(stayed.traveled, false);
  assert.equal(trailing.card.classList.has("is-arriving"), false);
});

test("newest-first insert at the top travels and reduced motion does not", () => {
  assert.ok(ARRIVAL_MS >= 200 && ARRIVAL_MS <= 300);
  const arriving = arrivalList("n", 0);
  const played = playQueueArrival(arriving.list, "n", { motion: true });
  assert.equal(played.traveled, true);
  assert.equal(played.duration, ARRIVAL_MS);
  assert.equal(arriving.card.classList.has("is-arriving"), true);
  assert.match(arriving.list.style.props["--arrive-shift"], /^-\d+px$/);
  arriving.card.classList.remove("is-arriving");
  const still = playQueueArrival(arriving.list, "n", { motion: false });
  assert.equal(still.traveled, false);
  assert.equal(arriving.card.classList.has("is-arriving"), false);
  assert.match(chrome, /--arrive:\s*280ms/);
  assert.match(chrome, /@keyframes bronze-arrive/);
  assert.match(chrome, /@keyframes bronze-make-room/);
  assert.match(
    chrome,
    /\.queue-item\.is-arriving\s*\{[^}]*animation:\s*bronze-arrive var\(--arrive\)/,
  );
  assert.match(
    chrome,
    /\[data-reduce-motion\] \.queue-item\.is-arriving[\s\S]*animation:\s*none/,
  );
});

test("notification click reveals a loaded card and loads a missing page by id", async () => {
  const scroller = {
    scrollTop: 100,
    getBoundingClientRect: () => ({ top: 0, bottom: 200 }),
    ownerDocument: { defaultView: null },
  };
  const card = {
    dataset: { itemId: "i1" },
    tabIndex: 0,
    focused: null,
    focus(opts) {
      this.focused = opts;
    },
    getBoundingClientRect: () => ({ top: -40, bottom: 10 }),
    closest: () => scroller,
  };
  const loaded = {
    dataset: { queueSort: "oldest" },
    querySelector(sel) {
      return sel === '[data-item-id="i1"]' ? card : null;
    },
  };
  const invokes = [];
  const shown = await revealQueueItem({
    id: "i1",
    list: loaded,
    doc: { documentElement: { hasAttribute: () => true } },
    invoke: async (name, args) => {
      invokes.push({ name, args });
    },
  });
  assert.equal(invokes.length, 0);
  assert.equal(shown.found, true);
  assert.equal(shown.behavior, "auto");
  assert.equal(scroller.scrollTop, 60);
  assert.equal(card.tabIndex, -1);
  assert.equal(card.focused.preventScroll, true);

  let visible = false;
  const inserted = [];
  const missing = {
    dataset: {},
    querySelector(sel) {
      if (!visible) {
        return null;
      }
      return sel === '[data-item-id="i9"]'
        ? {
            dataset: { itemId: "i9" },
            focus() {},
            scrollIntoView() {},
          }
        : null;
    },
  };
  const pageCalls = [];
  const located = await revealQueueItem({
    id: "i9",
    list: missing,
    doc: {
      documentElement: { hasAttribute: () => true },
    },
    invoke: async (name, args) => {
      pageCalls.push({ name, args });
      visible = true;
      return { items: [{ id: "i9" }, { id: "i10" }], index: 40 };
    },
    insertMissing(_list, items) {
      inserted.push(items.map((item) => item.id));
    },
  });
  assert.deepEqual(pageCalls, [
    {
      name: "queue_query",
      args: { filter: "overview", limit: 20, sort: "oldest", itemId: "i9" },
    },
  ]);
  assert.deepEqual(inserted, [["i9", "i10"]]);
  assert.equal(located.found, true);
  assert.equal(
    pageCalls.filter((call) => call.name !== "queue_query").length,
    0,
  );
});

test("reduce motion skips the travel animation", () => {
  assert.equal(
    scrollDelta({ top: 10, bottom: 40 }, { top: 0, bottom: 100 }),
    0,
  );
  assert.equal(
    scrollDelta({ top: -20, bottom: 30 }, { top: 0, bottom: 100 }),
    -20,
  );
  const frames = [];
  const scroller = { scrollTop: 0 };
  const still = travelScroll(scroller, 0, 80, {
    motion: false,
    frame(fn) {
      frames.push(fn);
      return 1;
    },
    now: () => 0,
  });
  assert.equal(still.behavior, "auto");
  assert.equal(frames.length, 0);
  assert.equal(scroller.scrollTop, 80);

  scroller.scrollTop = 0;
  const moving = travelScroll(scroller, 0, 80, {
    motion: true,
    duration: 180,
    frame(fn) {
      frames.push(fn);
      return 1;
    },
    now: () => 0,
  });
  assert.equal(moving.behavior, "smooth");
  assert.equal(frames.length, 1);
  frames[0](180);
  assert.equal(scroller.scrollTop, 80);
  assert.equal(frames.length, 1);
});

test("last copied card keeps a ring that reduce motion holds still", () => {
  const rows = [
    {
      dataset: { itemId: "a" },
      classList: toggleClass(),
    },
    {
      dataset: { itemId: "b" },
      classList: toggleClass(),
    },
  ];
  const list = {
    querySelectorAll() {
      return rows;
    },
  };
  markLastCopied(list, "b");
  assert.equal(rows[0].classList.has("is-last-copied"), false);
  assert.equal(rows[1].classList.has("is-last-copied"), true);
  assert.equal(rows[1].classList.has("is-ring-pulse"), false);
  markLastCopied(list, "b", { pulse: true });
  assert.equal(rows[1].classList.has("is-last-copied"), true);
  assert.equal(rows[1].classList.has("is-ring-pulse"), true);
  assert.equal(rows[0].classList.has("is-ring-pulse"), false);
  markLastCopied(list, "b");
  assert.equal(rows[1].classList.has("is-last-copied"), true);
  assert.equal(rows[1].classList.has("is-ring-pulse"), false);
  assert.match(chrome, /@keyframes bronze-copy-ring/);
  assert.match(chrome, /\.queue-item\.is-last-copied > article/);
  assert.match(
    chrome,
    /animation:\s*bronze-copy-ring var\(--arrive\) var\(--ease-out\) 3/,
  );
  assert.match(chrome, /\.is-ring-pulse > article/);
  assert.match(chrome, /border-radius:\s*var\(--radius-card\)/);
  assert.match(chrome, /0 0 0 8px color-mix\(in srgb, var\(--ring\)/);
  assert.match(
    chrome,
    /\[data-reduce-motion\] \.queue-item\.is-last-copied > article[\s\S]*animation:\s*none/,
  );
  assert.match(
    chrome,
    /\[data-reduce-motion\] \.queue-item\.is-last-copied > article[\s\S]*box-shadow:\s*0 0 0 2px var\(--ring\)/,
  );
  assert.match(
    chrome,
    /:focus-visible\s*\{[^}]*outline:\s*2px solid var\(--ring\)/,
  );
  assert.doesNotMatch(
    chrome,
    /\.queue-item\.is-last-copied[^}]*outline:\s*none/,
  );
  assert.match(html, /data-notice-reveal/);
  assert.match(html, /<button[^>]*id="chrome-notice-text"/);
  assert.doesNotMatch(html, /<div[^>]*data-notice-reveal/);
  assert.match(live, /planNewItemFollow/);
  assert.match(live, /plan\.scrollId/);
  assert.match(live, /playQueueArrival/);
  const follow = live.slice(live.indexOf("const plan = planNewItemFollow"));
  const travel = follow.slice(
    follow.indexOf("if (plan.scrollId"),
    follow.indexOf("if (lastCopiedId)"),
  );
  assert.match(travel, /playQueueArrival/);
  assert.doesNotMatch(travel, /markLastCopied/);
  assert.match(follow, /markLastCopied\(list, lastCopiedId\)/);
  assert.match(live, /markLastCopied\(list, id, \{[\s\S]*pulse: motionAllowed/);
  assert.match(reveal, /queue_query/);
  assert.match(reveal, /itemId/);
  assert.match(live, /itemId/);
  assert.doesNotMatch(reveal, /for \(let hop/);
  assert.doesNotMatch(reveal, /queue_page_for_item/);
  assert.doesNotMatch(reveal, /list_overview_items/);
  assert.match(live, /notice-activate/);
  assert.match(live, /markLastCopied/);
  assert.match(live, /ensureItemLoaded/);
  assert.doesNotMatch(live, /queue_page_for_item/);
  assert.doesNotMatch(live, /list_overview_items/);
  assert.doesNotMatch(live, /loadNextPage|advanceCursor|prependPage/);
});

function arrivalList(id, index) {
  const names = new Set();
  const card = {
    dataset: { itemId: id },
    classList: {
      add(name) {
        names.add(name);
      },
      remove(name) {
        names.delete(name);
      },
      has(name) {
        return names.has(name);
      },
    },
    getBoundingClientRect: () => ({ height: 80, top: 0, bottom: 80 }),
  };
  const head = {
    dataset: { itemId: "a" },
    classList: { add() {}, remove() {}, has() { return false; } },
  };
  const rows = index === 0 ? [card, head] : [head, card];
  return {
    card,
    list: {
      children: rows,
      style: {
        props: {},
        setProperty(name, value) {
          this.props[name] = value;
        },
        removeProperty(name) {
          delete this.props[name];
        },
      },
      querySelector(sel) {
        return sel === `[data-item-id="${id}"]` ? card : null;
      },
      ownerDocument: { defaultView: null },
    },
  };
}

function toggleClass() {
  const names = new Set();
  return {
    toggle(name, on) {
      if (on) {
        names.add(name);
      } else {
        names.delete(name);
      }
    },
    has(name) {
      return names.has(name);
    },
  };
}
