import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyQueueExitMotion,
  applyQueueItemMutation,
  createQueueRenderer,
  exitMotionClass,
  queueMotionKind,
  shouldAnimateQueue,
} from "./queue-motion.mjs";

const chrome = readFileSync(
  join(dirname(fileURLToPath(import.meta.url)), "chrome.css"),
  "utf8",
);

test("queue motion kind maps actions and list diffs", () => {
  assert.equal(
    queueMotionKind({
      prevIds: ["a", "b"],
      nextIds: ["b"],
      action: "complete",
    }),
    "exit",
  );
  assert.equal(
    queueMotionKind({ prevIds: ["a", "b"], nextIds: ["a"], action: "skip" }),
    "exit",
  );
  assert.equal(
    queueMotionKind({ prevIds: ["a"], nextIds: [], action: "trash" }),
    "exit",
  );
  assert.equal(
    queueMotionKind({
      prevIds: ["a", "b"],
      nextIds: ["b", "a"],
      action: "moveUp",
    }),
    "move",
  );
  assert.equal(
    queueMotionKind({
      prevIds: ["a"],
      nextIds: ["n", "a"],
      action: "insert",
    }),
    "enter",
  );
  assert.equal(
    queueMotionKind({ prevIds: ["a"], nextIds: ["a"], action: "replace" }),
    "replace",
  );
  assert.equal(
    queueMotionKind({ prevIds: ["a", "b"], nextIds: ["b"] }),
    "exit",
  );
  assert.equal(
    queueMotionKind({ prevIds: ["a", "b"], nextIds: ["b", "a"] }),
    "move",
  );
  assert.equal(exitMotionClass("complete"), "is-leaving-complete");
  assert.equal(exitMotionClass("skip"), "is-leaving-skip");
  assert.equal(exitMotionClass("trash"), "is-leaving-trash");
  assert.equal(
    shouldAnimateQueue(
      {
        documentElement: { hasAttribute: () => true },
      },
      "exit",
    ),
    false,
  );
});

test("reduced motion exit does not wait and persist still runs", async () => {
  const reduceDoc = {
    documentElement: { hasAttribute: () => true },
    defaultView: {
      matchMedia: () => ({ matches: true }),
      setTimeout() {
        throw new Error("reduced motion must not wait");
      },
    },
  };
  const added = [];
  const node = {
    classList: {
      add(name) {
        added.push(name);
      },
    },
    ownerDocument: reduceDoc,
    addEventListener() {
      throw new Error("reduced motion must not wait");
    },
  };
  const started = Date.now();
  const result = await applyQueueExitMotion(node, "complete");
  assert.equal(result.animated, false);
  assert.equal(result.className, null);
  assert.deepEqual(added, []);
  assert.ok(Date.now() - started < 40);
  const order = [];
  await applyQueueItemMutation(
    async (name) => {
      order.push(name);
    },
    { id: "x", action: "complete" },
    async () => {
      order.push("refresh");
    },
  );
  assert.deepEqual(order, ["apply_queue_item_action", "refresh"]);
  const recovered = [];
  await applyQueueItemMutation(
    async (name) => {
      recovered.push(name);
    },
    { id: "x", action: "skip" },
    async () => {
      recovered.push("refresh");
      throw new Error("motion failed");
    },
  );
  assert.deepEqual(recovered, [
    "apply_queue_item_action",
    "refresh",
    "refresh",
  ]);
});

test("motion exit applies the leaving class", async () => {
  const added = [];
  const node = {
    classList: {
      add(name) {
        added.push(name);
      },
    },
    ownerDocument: {
      documentElement: { hasAttribute: () => false },
      defaultView: {
        matchMedia: () => ({ matches: false }),
        setTimeout(fn) {
          fn();
          return 1;
        },
      },
    },
    addEventListener() {},
    removeEventListener() {},
  };
  const result = await applyQueueExitMotion(node, "complete");
  assert.equal(result.animated, true);
  assert.equal(result.className, "is-leaving-complete");
  assert.deepEqual(added, ["is-leaving-complete"]);
  const skip = await applyQueueExitMotion(
    {
      ...node,
      classList: {
        add(name) {
          added.push(name);
        },
      },
    },
    "skip",
  );
  assert.equal(skip.className, "is-leaving-skip");
  const trash = await applyQueueExitMotion(
    {
      ...node,
      classList: {
        add(name) {
          added.push(name);
        },
      },
    },
    "trash",
  );
  assert.equal(trash.className, "is-leaving-trash");
});

test("a later refresh keeps the sliding row instead of replacing it", async () => {
  const list = fakeQueueList();
  const template = fakeQueueTemplate();
  const renderer = createQueueRenderer({
    queueItemRows(target) {
      return target.children;
    },
    syncMoveAvailability() {},
  });
  await renderer.renderQueueItems(list, [{ id: "n", body: "new" }], template, {
    action: "insert",
  });
  assert.equal(list.children.length, 1);
  const card = list.children[0];
  assert.equal(card.classList.has("is-slide-in"), true);
  assert.equal(card.classList.has("is-entering"), false);
  await renderer.renderQueueItems(
    list,
    [
      { id: "n", body: "new", title: "Ring and Motion Problem" },
      { id: "n", body: "new", title: "duplicate" },
    ],
    template,
    { action: "replace" },
  );
  assert.equal(list.children.length, 1);
  assert.equal(list.children[0], card);
  assert.equal(card.classList.has("is-slide-in"), true);
  assert.equal(list.relocations, 0);
  const removed = [];
  const originalRemove = card.classList.remove.bind(card.classList);
  card.classList.remove = (name) => {
    removed.push(name);
    originalRemove(name);
  };
  await renderer.renderQueueItems(
    list,
    [{ id: "n", body: "new", title: "still sliding" }],
    template,
    { action: "replace" },
  );
  assert.equal(list.children[0], card);
  assert.equal(card.classList.has("is-slide-in"), true);
  assert.equal(removed.includes("is-slide-in"), false);
  assert.equal(list.relocations, 0);
});

test("a new card is inserted once and neighbors stay attached", async () => {
  const list = fakeQueueList();
  const template = fakeQueueTemplate();
  const renderer = createQueueRenderer({
    queueItemRows(target) {
      return target.children;
    },
    syncMoveAvailability() {},
  });
  await renderer.renderQueueItems(
    list,
    [
      { id: "a", body: "old" },
      { id: "b", body: "older" },
    ],
    template,
  );
  const old = list.children[0];
  const older = list.children[1];
  const parked = list.relocations;
  await renderer.renderQueueItems(
    list,
    [
      { id: "n", body: "new" },
      { id: "a", body: "old" },
      { id: "b", body: "older" },
    ],
    template,
    { action: "insert", slide: true },
  );
  assert.equal(list.children[0].dataset.itemId, "n");
  assert.equal(list.children[0].classList.has("is-slide-in"), true);
  assert.equal(old.classList.has("is-slide-in"), false);
  assert.equal(older.classList.has("is-slide-in"), false);
  assert.equal(old.classList.has("is-ring-pulse"), false);
  assert.equal(older.classList.has("is-ring-pulse"), false);
  assert.equal(list.classList.has("is-slide-in"), false);
  assert.equal(list.classList.has("is-arriving"), false);
  assert.equal(list.children[1], old);
  assert.equal(list.children[2], older);
  assert.equal(list.relocations, parked);
  await renderer.renderQueueItems(
    list,
    [
      { id: "a", body: "old" },
      { id: "b", body: "paged" },
      { id: "c", body: "paged" },
    ],
    template,
    { action: "insert", slide: false },
  );
  const paged = list.children.filter((node) => node.dataset.itemId !== "a");
  assert.equal(
    paged.some((node) => node.classList.has("is-slide-in")),
    false,
  );
});

test("a title update keeps the row and the reserved title-box height", async () => {
  const list = fakeQueueList();
  const template = fakeQueueTemplate();
  const renderer = createQueueRenderer({
    queueItemRows(target) {
      return target.children;
    },
    syncMoveAvailability() {},
  });
  const slot = queueTitleSlotRule(chrome);
  const oneLine = "Thanks.";
  const twoLine =
    "The generated title wraps onto a second line of this queue card";
  await renderer.renderQueueItems(
    list,
    [{ id: "n", title: oneLine, body: "body" }],
    template,
    { action: "insert" },
  );
  assert.equal(list.children.length, 1);
  const card = list.children[0];
  const firstHeight = reservedTitleBoxHeight(oneLine, slot);
  const replacesAfterInsert = list.replaceCount;
  await renderer.renderQueueItems(
    list,
    [{ id: "n", title: twoLine, body: "body" }],
    template,
    { action: "replace" },
  );
  assert.equal(list.children[0], card);
  assert.equal(list.replaceCount, replacesAfterInsert);
  assert.equal(reservedTitleBoxHeight(twoLine, slot), firstHeight);
  assert.equal(reservedTitleBoxHeight("Hi", slot), firstHeight);
  assert.equal(firstHeight, 2);
});

function queueTitleSlotRule(css) {
  const match = css.match(
    /#queue article \[data-slot="title"\]:not\(\[hidden\]\)\s*\{([^}]+)\}/,
  );
  assert.ok(match, "queue title reserved slot");
  return match[1];
}

function reservedTitleBoxHeight(text, rule) {
  const clampMatch = rule.match(/-webkit-line-clamp:\s*(\d+)/);
  const clamp = Number(clampMatch?.[1] ?? Number.POSITIVE_INFINITY);
  const minMatch = rule.match(/min-height:\s*calc\(\s*(\d+)\s*\*\s*1lh\s*\)/);
  const minLines = Number(minMatch?.[1] ?? 0);
  const natural = Math.max(1, Math.ceil(String(text).length / 24));
  return Math.max(Math.min(natural, clamp), minLines);
}

function fakeQueueList() {
  const children = [];
  const list = {
    children,
    relocations: 0,
    classList: classBag(),
    replaceCount: 0,
    ownerDocument: {
      documentElement: {
        dataset: { motion: "full" },
        hasAttribute() {
          return false;
        },
      },
    },
    append(node) {
      detach(node);
      children.push(node);
      own(node);
    },
    insertBefore(node, before) {
      detach(node);
      const at = children.indexOf(before);
      if (at < 0) {
        children.push(node);
      } else {
        children.splice(at, 0, node);
      }
      own(node);
    },
    replaceChildren() {
      list.replaceCount += 1;
      children.length = 0;
    },
  };
  function detach(node) {
    const index = children.indexOf(node);
    if (index < 0) {
      return;
    }
    children.splice(index, 1);
    list.relocations += 1;
  }
  function own(node) {
    node.remove = () => {
      const index = children.indexOf(node);
      if (index >= 0) {
        children.splice(index, 1);
      }
    };
  }
  return list;
}

function fakeQueueTemplate() {
  return {
    content: {
      querySelector() {
        return null;
      },
      firstElementChild: {
        cloneNode() {
          return {
            dataset: {},
            classList: classBag(),
            querySelector() {
              return null;
            },
            querySelectorAll() {
              return [];
            },
            remove() {},
          };
        },
      },
    },
  };
}

function classBag() {
  const names = new Set();
  return {
    add(name) {
      names.add(name);
    },
    remove(name) {
      names.delete(name);
    },
    has(name) {
      return names.has(name);
    },
  };
}
