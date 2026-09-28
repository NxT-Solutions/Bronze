import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyMotionDataset,
  reduceMotionFromSettings,
} from "./apply-motion.mjs";
import { motionAllowed } from "./control.mjs";
import {
  ARRIVE_ROOM_MS,
  ARRIVE_SLIDE_DELAY_MS,
  ARRIVE_SLIDE_MS,
  beginSortCollapse,
  beginSortExpand,
  insertionBeforeId,
  isNearLoadedStart,
  markLastCopied,
  normalizeQueueSort,
  planNewItemFollow,
  presentNewQueueCard,
  revealQueueItem,
  scrollDelta,
  shouldLoadNextOnKey,
  SORT_COLLAPSE_MS,
  SORT_EXPAND_MS,
  sortReflowPlan,
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

test("newest-first follows a card at the loaded start", () => {
  const prev = ["a", "b"];
  const atTop = planNewItemFollow({
    sort: "newest",
    nearStart: true,
    prevIds: prev,
    nextIds: ["n", "a", "b"],
  });
  assert.deepEqual(atTop, {
    scrollId: "n",
    cursor: "stay",
    prepend: false,
    room: "start",
  });
  assert.equal(normalizeQueueSort("created-desc"), "newest");
  assert.equal(isNearLoadedStart({ scrollTop: 0 }), true);
  assert.equal(isNearLoadedStart({ scrollTop: 80 }), false);

  const scrolledAway = planNewItemFollow({
    sort: "newest",
    nearStart: false,
    prevIds: prev,
    nextIds: ["n", "a", "b"],
  });
  assert.equal(scrolledAway.scrollId, "n");
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
  assert.equal(plan.room, "end");
  assert.equal(
    planNewItemFollow({
      sort: "oldest",
      prevIds: ["a", "b"],
      nextIds: ["a", "b", "n"],
      hasMore: true,
    }).room,
    null,
  );
  assert.equal(normalizeQueueSort("rank"), "oldest");
  assert.equal(normalizeQueueSort(""), "oldest");
  const endNeighbor = insertionBeforeId(["a", "b"], ["a", "b", "n"], "n");
  assert.equal(endNeighbor, null);
  assert.equal(insertionBeforeId(["b"], ["n", "b"], "n"), "b");
  const writes = [];
  const scroller = scrollProbe(40, writes);
  const list = attachRows(slideCard(96), slideCard(96));
  const last = list.children[1];
  const trailing = slideCard(96);
  list.append(trailing);
  const stayed = presentNewQueueCard(trailing, scroller, {
    motion: true,
    followScroll: false,
    pulse: true,
    duration: 340,
    frame(fn) {
      writes.push(["frame", fn]);
      return 1;
    },
    now: () => 0,
  });
  assert.equal(stayed.scrolled, false);
  assert.equal(scroller.scrollTop, 40);
  assert.equal(writes.filter((entry) => typeof entry === "number").length, 0);
  assert.equal(trailing.classList.has("is-slide-in"), true);
  assert.equal(trailing.classList.has("is-slot-pending"), true);
  assert.equal(last.classList.has("is-make-room"), true);
  assert.equal(last.classList.has("is-make-room-end"), true);
  assert.equal(trailing.classList.has("is-ring-pulse"), false);
});

test("newest-first at the top slides the new card without moving the scrollport", () => {
  const writes = [];
  const scroller = scrollProbe(0, writes);
  const card = slideCard(96);
  const older = slideCard(96);
  attachRows(card, older);
  const played = presentNewQueueCard(card, scroller, {
    motion: true,
    followScroll: true,
    pulse: true,
    duration: 340,
    frame() {
      throw new Error("already at the top");
    },
    now: () => 0,
  });
  assert.equal(card.classList.has("is-slide-in"), true);
  assert.equal(card.classList.has("is-ring-pulse"), false);
  assert.equal(card.classList.has("is-last-copied"), false);
  assert.equal(older.classList.has("is-make-room"), true);
  assert.equal(older.classList.has("is-make-room-end"), false);
  assert.equal(older.classList.has("is-slide-in"), false);
  assert.equal(older.classList.has("is-ring-pulse"), false);
  assert.equal(played.scrolled, false);
  assert.equal(played.from, 0);
  assert.equal(played.to, 0);
  assert.equal(scroller.scrollTop, 0);
  assert.equal(writes.length, 0);
  assert.equal(ARRIVE_ROOM_MS, 320);
  assert.equal(ARRIVE_SLIDE_DELAY_MS, 80);
  assert.equal(ARRIVE_SLIDE_MS, 380);
  assert.match(chrome, /--arrive:\s*380ms/);
  assert.match(chrome, /--arrive-room:\s*320ms/);
  assert.match(chrome, /--arrive-scroll:\s*340ms/);
  assert.match(chrome, /--sort-flip:\s*200ms/);
  assert.match(chrome, /--sort-collapse:\s*220ms/);
  assert.match(chrome, /--sort-expand:\s*280ms/);
  assert.match(
    chrome,
    /\.queue-sort-toggle \[data-sort-glyph\][\s\S]*transition:[\s\S]*transform var\(--sort-flip\)/,
  );
  assert.doesNotMatch(
    chrome,
    /\.queue-sort-toggle\[data-queue-sort="newest"\] \[data-sort-glyph="oldest"\][\s\S]{0,80}display:\s*none/,
  );
  assert.match(chrome, /#queue\.is-sort-reflow/);
  assert.match(chrome, /@keyframes bronze-sort-collapse/);
  assert.match(chrome, /@keyframes bronze-sort-expand/);
  assert.match(chrome, /@keyframes bronze-sort-expand-lead/);
  assert.match(
    chrome,
    /\.queue-item\.is-sort-collapse[\s\S]*bronze-sort-collapse var\(--sort-collapse\)/,
  );
  assert.match(
    chrome,
    /\.queue-item\.is-sort-expand[\s\S]*bronze-sort-expand var\(--sort-expand\)/,
  );
  assert.doesNotMatch(
    chrome,
    /#queue\.is-sort-reflow[\s\S]{0,120}translateX/,
  );
  assert.match(chrome, /@keyframes bronze-slide-in/);
  assert.match(chrome, /translateX\(-100%\)/);
  assert.match(chrome, /@keyframes bronze-slide-in-rtl/);
  assert.match(chrome, /translateX\(100%\)/);
  assert.match(chrome, /@keyframes bronze-make-room/);
  assert.match(
    chrome,
    /translateY\(calc\(-1 \* var\(--arrive-block, 0px\)\)\)/,
  );
  assert.match(chrome, /@keyframes bronze-make-room-end/);
  assert.match(
    chrome,
    /\.queue-item\.is-slide-in\s*\{[^}]*animation:\s*bronze-slide-in var\(--arrive\)/,
  );
  assert.doesNotMatch(
    chrome,
    /\.queue-item\.is-slide-in:not\(\.is-slide-docked\)/,
  );
  assert.doesNotMatch(
    chrome,
    /margin-block-end:\s*calc\(-1 \* var\(--arrive-block/,
  );
  assert.match(chrome, /#queue\.is-slide-in[\s\S]*animation:\s*none/);
  assert.doesNotMatch(chrome, /@keyframes bronze-arrive/);
  assert.match(
    chrome,
    /\[data-reduce-motion\] \.queue-item\.is-slide-in[\s\S]*animation:\s*none/,
  );
  assert.match(
    chrome,
    /\[data-reduce-motion\] \.queue-item\.is-make-room[\s\S]*animation:\s*none/,
  );
});

test("the queue list element does not receive the slide class", () => {
  const list = {
    id: "queue",
    dataset: {},
    classList: classNames(),
  };
  const played = presentNewQueueCard(list, null, {
    motion: true,
    followScroll: true,
    pulse: true,
  });
  assert.equal(list.classList.has("is-slide-in"), false);
  assert.equal(list.classList.has("is-arriving"), false);
  assert.equal(list.classList.has("is-ring-pulse"), false);
  assert.equal(played.scrolled, false);
});

test("play animations applies the slide and the pulse", () => {
  assert.equal(
    reduceMotionFromSettings({ general: { reduceMotion: "off" } }),
    "off",
  );
  const attrs = new Set();
  const html = {
    dataset: {},
    toggleAttribute(name, on) {
      if (on) {
        attrs.add(name);
      } else {
        attrs.delete(name);
      }
    },
  };
  const doc = {
    documentElement: html,
    defaultView: { matchMedia: () => ({ matches: true }) },
  };
  applyMotionDataset(html, "off", true);
  assert.equal(html.dataset.motion, "full");
  assert.equal(motionAllowed(doc), true);
  const writes = [];
  const frames = [];
  const card = slideCard(80);
  const played = presentNewQueueCard(card, scrollProbe(80, writes), {
    motion: motionAllowed(doc),
    followScroll: true,
    pulse: true,
    duration: 340,
    frame(fn) {
      frames.push(fn);
      return 1;
    },
    now: () => 0,
  });
  assert.equal(card.classList.has("is-slide-in"), true);
  assert.equal(card.classList.has("is-ring-pulse"), false);
  assert.equal(played.behavior, "smooth");
  assert.equal(writes.length, 0);
  frames[0](0);
  assert.equal(writes[0], 80);
  frames[frames.length - 1](170);
  assert.ok(writes.length > 1);
  assert.ok(writes.some((value) => value > 0 && value < 80));
  frames[frames.length - 1](340);
  assert.equal(writes.at(-1), 0);
  assert.ok(writes.indexOf(0) > 0);

  applyMotionDataset(html, "on", false);
  assert.equal(html.dataset.motion, "reduce");
  assert.equal(motionAllowed(doc), false);
  const reducedWrites = [];
  const reduced = slideCard(80);
  presentNewQueueCard(reduced, scrollProbe(0, reducedWrites), {
    motion: motionAllowed(doc),
    followScroll: true,
    pulse: true,
    frame() {
      throw new Error("reduced motion must not scroll");
    },
  });
  assert.equal(reduced.classList.has("is-slide-in"), false);
  assert.equal(reduced.classList.has("is-ring-pulse"), false);
  assert.equal(reduced.classList.has("is-last-copied"), false);
  assert.equal(reducedWrites.length, 0);

  const system = {
    documentElement: {
      dataset: {},
      hasAttribute: () => false,
    },
    defaultView: { matchMedia: () => ({ matches: true }) },
  };
  assert.equal(motionAllowed(system), false);
  const systemCard = slideCard(40);
  presentNewQueueCard(systemCard, scrollProbe(12, []), {
    motion: motionAllowed(system),
    followScroll: true,
    pulse: true,
  });
  assert.equal(systemCard.classList.has("is-slide-in"), false);
  assert.equal(systemCard.classList.has("is-ring-pulse"), false);
  assert.equal(attrs.has("data-reduce-motion"), true);
  assert.match(
    chrome,
    /html\[data-motion="reduce"\] \.queue-sort-toggle \[data-sort-glyph\][\s\S]*transition:\s*none/,
  );
});

test("reduced motion inserts the card without motion classes", () => {
  const writes = [];
  const scroller = scrollProbe(0, writes);
  const card = slideCard(96);
  const older = slideCard(96);
  attachRows(card, older);
  const still = presentNewQueueCard(card, scroller, {
    motion: false,
    followScroll: true,
    pulse: true,
    duration: 340,
    frame() {
      throw new Error("scroll frame");
    },
    now: () => 0,
  });
  assert.equal(still.scrolled, false);
  assert.equal(scroller.scrollTop, 0);
  assert.equal(writes.length, 0);
  assert.equal(card.classList.has("is-slide-in"), false);
  assert.equal(card.classList.has("is-make-room"), false);
  assert.equal(older.classList.has("is-make-room"), false);
  assert.equal(older.classList.has("is-slide-in"), false);
  assert.equal(card.classList.has("is-ring-pulse"), false);
  assert.equal(card.classList.has("is-entering"), false);
  assert.equal(card.classList.has("is-last-copied"), false);
});

test("the arrival ring class leaves when the pulse ends", () => {
  const timers = [];
  const events = [];
  const article = {
    addEventListener(type, fn) {
      events.push({ type, fn });
    },
  };
  const card = slideCard(40);
  card.querySelector = (sel) => (sel === "article" ? article : null);
  card.ownerDocument = {
    defaultView: {
      setTimeout(fn, ms) {
        timers.push({ fn, ms });
        return 1;
      },
    },
  };
  presentNewQueueCard(card, null, { motion: true, pulse: true });
  assert.equal(card.classList.has("is-slide-in"), true);
  assert.equal(card.classList.has("is-ring-pulse"), false);
  assert.equal(card.classList.has("is-last-copied"), false);
  const landTimers = timers.filter((timer) => timer.ms === ARRIVE_SLIDE_MS);
  assert.ok(landTimers.length > 0);
  for (const timer of landTimers) {
    timer.fn();
  }
  assert.equal(card.classList.has("is-slide-in"), false);
  assert.equal(card.classList.has("is-slide-docked"), false);
  assert.equal(card.classList.has("is-ring-pulse"), true);
  const ringTimer = timers.find((timer) => timer.ms === 1000);
  assert.ok(ringTimer);
  events[0].fn({ animationName: "bronze-copy-ring" });
  assert.equal(card.classList.has("is-ring-pulse"), false);

  const held = slideCard(40);
  const later = [];
  held.ownerDocument = {
    defaultView: {
      setTimeout(fn, ms) {
        later.push({ fn, ms });
        return 1;
      },
    },
  };
  presentNewQueueCard(held, null, { motion: true, pulse: true });
  assert.equal(held.classList.has("is-ring-pulse"), false);
  for (const timer of later.filter((entry) => entry.ms === ARRIVE_SLIDE_MS)) {
    timer.fn();
  }
  assert.equal(held.classList.has("is-ring-pulse"), true);
  later.find((timer) => timer.ms === 1000).fn();
  assert.equal(held.classList.has("is-ring-pulse"), false);
  assert.equal(held.classList.has("is-slide-in"), false);
});

test("make-room and slide classes leave no leftover transform or margin", () => {
  const timers = [];
  const older = slideCard(96);
  const card = slideCard(96);
  const list = attachRows(card, older);
  list.ownerDocument = {
    defaultView: {
      setTimeout(fn, ms) {
        timers.push({ fn, ms });
        return 1;
      },
    },
  };
  card.ownerDocument = list.ownerDocument;
  older.ownerDocument = list.ownerDocument;
  presentNewQueueCard(card, null, { motion: true, pulse: true });
  assert.equal(card.classList.has("is-slide-in"), true);
  assert.equal(older.classList.has("is-make-room"), true);
  assert.equal(list.style.getPropertyValue("--arrive-block"), "112px");
  for (const timer of timers.filter((entry) => entry.ms === ARRIVE_ROOM_MS)) {
    timer.fn();
  }
  assert.equal(older.classList.has("is-make-room"), false);
  assert.equal(older.style.transform, "");
  assert.equal(older.style.margin, "");
  const slideAt = ARRIVE_SLIDE_DELAY_MS + ARRIVE_SLIDE_MS;
  for (const timer of timers.filter((entry) => entry.ms === slideAt)) {
    timer.fn();
  }
  assert.equal(card.classList.has("is-slide-in"), false);
  assert.equal(card.style.transform, "");
  assert.equal(card.style.margin, "");
  assert.equal(card.style.getPropertyValue("--arrive-slide-delay"), "");
  assert.equal(list.style.getPropertyValue("--arrive-block"), "");
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

test("the copied pulse is temporary and is not the focus ring", () => {
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
  assert.equal(rows[1].classList.has("is-ring-pulse"), true);
  markLastCopied(list, "a", { pulse: true });
  assert.equal(rows[1].classList.has("is-ring-pulse"), false);
  assert.equal(rows[0].classList.has("is-ring-pulse"), true);
  assert.match(chrome, /@keyframes bronze-copy-ring/);
  assert.match(chrome, /outline-color:\s*transparent/);
  assert.match(
    chrome,
    /outline-color:\s*color-mix\(in srgb, var\(--ring\) 40%, transparent\)/,
  );
  assert.match(chrome, /outline-offset:\s*2px/);
  assert.match(chrome, /--ring-pulse:\s*1000ms/);
  assert.match(
    chrome,
    /\.queue-item\.is-ring-pulse > article\s*\{[^}]*animation:\s*bronze-copy-ring calc\(var\(--ring-pulse\) \/ 3\) var\(--ease-out\) 3 both/,
  );
  assert.match(chrome, /outline-offset:\s*4px/);
  assert.match(chrome, /\.is-ring-pulse > article/);
  assert.match(chrome, /border-radius:\s*var\(--radius-card\)/);
  assert.doesNotMatch(
    chrome,
    /\.queue-item\.is-(?:last-copied|ring-pulse) > article\s*\{[^}]*outline:\s*2px solid var\(--ring\)/,
  );
  assert.match(
    chrome,
    /\[data-reduce-motion\] \.queue-item\.is-ring-pulse > article[\s\S]*?outline:\s*none/,
  );
  assert.doesNotMatch(
    chrome,
    /\[data-reduce-motion\] \.queue-item\.is-ring-pulse > article[\s\S]*?outline:\s*2px solid var\(--ring\)/,
  );
  assert.match(
    chrome,
    /:focus-visible\s*\{\s*outline:\s*2px solid var\(--ring\);\s*outline-offset:\s*2px;\s*\}/,
  );
  assert.doesNotMatch(chrome, /:focus-visible\s*\{[^}]*outline:\s*none/);
  assert.match(html, /data-notice-reveal/);
  assert.match(html, /<button[^>]*id="chrome-notice-text"/);
  assert.doesNotMatch(html, /<div[^>]*data-notice-reveal/);
  assert.match(live, /planNewItemFollow/);
  assert.match(live, /plan\.scrollId/);
  assert.match(live, /presentNewQueueCard/);
  assert.match(live, /followScroll: id === edgeId/);
  assert.match(reveal, /is-slide-in/);
  assert.doesNotMatch(reveal, /primeStart:\s*true/);
  const follow = live.slice(live.indexOf("const plan = planNewItemFollow"));
  assert.match(follow, /hasMore/);
  assert.match(live, /is-sort-reflow/);
  assert.match(live, /beginSortCollapse/);
  assert.match(live, /beginSortExpand/);
  assert.match(live, /sortReflowPlan/);
  assert.match(live, /sortChanged/);
  assert.match(follow, /presentNewQueueCard/);
  assert.match(follow, /pulse: added\.length === 1/);
  assert.match(follow, /markLastCopied\(list, added\[0\]/);
  assert.doesNotMatch(follow, /scroller\.scrollTop\s*=/);
  assert.match(
    follow,
    /markLastCopied\(list, lastCopiedId(?:, \{[\s\S]*pulse:)/,
  );
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

test("sort reflow collapses then expands from the first card", () => {
  assert.equal(SORT_COLLAPSE_MS + SORT_EXPAND_MS, 500);
  assert.deepEqual(sortReflowPlan(true, 3), { collapse: true, expand: true });
  assert.deepEqual(sortReflowPlan(false, 3), {
    collapse: false,
    expand: false,
  });
  assert.deepEqual(sortReflowPlan(true, 0), { collapse: false, expand: false });

  const first = slideCard(80);
  const second = slideCard(80);
  second.getBoundingClientRect = () => ({ height: 80, top: 96, bottom: 176 });
  const list = attachRows(first, second);
  list.classList = classNames();
  const order = [];
  const add = list.classList.add.bind(list.classList);
  list.classList.add = (name) => {
    order.push(name);
    add(name);
  };

  const collapse = beginSortCollapse(list);
  assert.equal(collapse.played, true);
  assert.equal(collapse.durationMs, SORT_COLLAPSE_MS);
  assert.equal(list.classList.has("is-sort-collapse"), true);
  assert.equal(list.classList.has("is-sort-expand"), false);
  assert.equal(first.classList.has("is-sort-collapse"), true);
  assert.equal(first.classList.has("is-sort-anchor"), true);
  assert.equal(second.classList.has("is-sort-collapse"), true);
  assert.equal(second.style.getPropertyValue("--sort-shift"), "96px");

  const expand = beginSortExpand(list);
  assert.equal(expand.played, true);
  assert.equal(expand.durationMs, SORT_EXPAND_MS);
  assert.equal(list.classList.has("is-sort-collapse"), false);
  assert.equal(list.classList.has("is-sort-expand"), true);
  assert.equal(first.classList.has("is-sort-lead"), true);
  assert.equal(first.classList.has("is-sort-collapse"), false);
  assert.deepEqual(
    order.filter((name) => name === "is-sort-collapse" || name === "is-sort-expand"),
    ["is-sort-collapse", "is-sort-expand"],
  );
  assert.match(
    chrome,
    /html\[data-motion="reduce"\] \.queue-item\.is-sort-collapse[\s\S]*animation:\s*none/,
  );
});

function classNames() {
  const names = new Set();
  return {
    add(name) {
      names.add(name);
    },
    remove(name) {
      names.delete(name);
    },
    toggle(name, on) {
      if (on) {
        names.add(name);
      } else {
        names.delete(name);
      }
    },
    contains(name) {
      return names.has(name);
    },
    has(name) {
      return names.has(name);
    },
  };
}

function styleBag() {
  const props = new Map();
  const style = {
    transform: "",
    margin: "",
    marginBlockStart: "",
    marginBlockEnd: "",
    setProperty(name, value) {
      props.set(name, String(value));
    },
    removeProperty(name) {
      props.delete(name);
      if (name === "transform") {
        style.transform = "";
      }
      if (
        name === "margin" ||
        name === "margin-block-start" ||
        name === "margin-block-end"
      ) {
        style.margin = "";
        style.marginBlockStart = "";
        style.marginBlockEnd = "";
      }
    },
    getPropertyValue(name) {
      return props.get(name) ?? "";
    },
  };
  return style;
}

function slideCard(height) {
  return {
    dataset: {},
    classList: classNames(),
    style: styleBag(),
    parentElement: null,
    nextElementSibling: null,
    previousElementSibling: null,
    getBoundingClientRect: () => ({ height, top: 0, bottom: height }),
  };
}

function attachRows(...cards) {
  const children = [];
  const list = {
    children,
    style: styleBag(),
    get firstElementChild() {
      return children[0] ?? null;
    },
    get lastElementChild() {
      return children[children.length - 1] ?? null;
    },
    append(node) {
      const index = children.indexOf(node);
      if (index >= 0) {
        children.splice(index, 1);
      }
      children.push(node);
      relink();
    },
  };
  function relink() {
    for (let index = 0; index < children.length; index += 1) {
      const node = children[index];
      node.parentElement = list;
      node.nextElementSibling = children[index + 1] ?? null;
      node.previousElementSibling = children[index - 1] ?? null;
    }
  }
  for (const card of cards) {
    list.append(card);
  }
  return list;
}

function scrollProbe(start, writes) {
  let top = start;
  return {
    get scrollTop() {
      return top;
    },
    set scrollTop(value) {
      writes.push(value);
      top = value;
    },
    ownerDocument: { defaultView: null },
  };
}

function toggleClass() {
  return classNames();
}
