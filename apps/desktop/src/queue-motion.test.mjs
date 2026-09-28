import assert from "node:assert/strict";
import { test } from "node:test";
import {
  applyQueueExitMotion,
  applyQueueItemMutation,
  createQueueRenderer,
  exitMotionClass,
  queueMotionKind,
  shouldAnimateQueue,
} from "./queue-motion.mjs";

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
  await renderer.renderQueueItems(
    list,
    [{ id: "n", body: "new" }],
    template,
    { action: "insert" },
  );
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
});

function fakeQueueList() {
  const children = [];
  return {
    children,
    ownerDocument: {
      documentElement: {
        dataset: { motion: "full" },
        hasAttribute() {
          return false;
        },
      },
    },
    append(node) {
      const index = children.indexOf(node);
      if (index >= 0) {
        children.splice(index, 1);
      }
      children.push(node);
    },
    replaceChildren() {
      children.length = 0;
    },
  };
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
