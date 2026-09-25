import assert from "node:assert/strict";
import { test } from "node:test";
import { motionAllowed } from "./control.mjs";
import {
  markLastCopied,
  openCopiedNotice,
  planNewItemFollow,
  shouldLoadNextOnKey,
  travelScroll,
} from "./queue-reveal.mjs";

test("SET-001 autoscroll follows a new copy only for newest-first at the top", () => {
  assert.equal(
    planNewItemFollow({
      sort: "newest",
      nearStart: true,
      prevIds: ["b", "a"],
      nextIds: ["c", "b", "a"],
    }).scrollId,
    "c",
  );
  assert.equal(
    planNewItemFollow({
      sort: "newest",
      nearStart: false,
      prevIds: ["b", "a"],
      nextIds: ["c", "b", "a"],
    }).scrollId,
    null,
  );
  assert.equal(
    planNewItemFollow({
      sort: "oldest",
      nearStart: true,
      prevIds: ["a", "b"],
      nextIds: ["a", "b", "c"],
    }).scrollId,
    null,
  );
});

test("A11Y-001 keyboard at the loaded end requests the next cursor page", () => {
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
      key: "ArrowDown",
      onLastCard: true,
      hasCursor: false,
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
});

test("copied notice resolves an item that is not on the first page", async () => {
  const first = [{ id: "a" }, { id: "b" }];
  const located = [{ id: "a" }, { id: "b" }, { id: "c" }, { id: "d" }];
  const result = await openCopiedNotice({
    id: "d",
    loaded: first,
    sort: "newest",
    invoke: async (name, args) => {
      assert.equal(name, "queue_page_for_item");
      assert.equal(args.id, "d");
      assert.equal(args.sort, "newest");
      return { items: located, nextCursor: "v1.next" };
    },
  });
  assert.equal(result.fetched, true);
  assert.equal(result.found, true);
  assert.ok(result.items.length > first.length);
  assert.equal(result.behavior, "auto");
});

test("A11Y-003 reduced motion skips travel to the copied card", () => {
  const frames = [];
  const scroller = { scrollTop: 40 };
  const jumped = travelScroll(scroller, 40, 240, {
    motion: false,
    frame: (step) => frames.push(step),
  });
  assert.equal(jumped.behavior, "auto");
  assert.equal(scroller.scrollTop, 240);
  assert.equal(frames.length, 0);
  const moved = { scrollTop: 10 };
  const played = travelScroll(moved, 10, 80, {
    motion: true,
    duration: 100,
    now: () => 0,
    frame: (step) => step(100),
  });
  assert.equal(played.behavior, "smooth");
  assert.equal(moved.scrollTop, 80);
  assert.equal(
    motionAllowed({ documentElement: { dataset: { motion: "reduce" } } }),
    false,
  );
  assert.equal(
    motionAllowed({
      documentElement: { dataset: { motion: "full" } },
      defaultView: { matchMedia: () => ({ matches: true }) },
    }),
    true,
  );
});

test("last-copied ring marks one card", () => {
  const label = { hidden: true };
  const row = {
    dataset: { itemId: "d" },
    classList: {
      on: false,
      toggle(_name, value) {
        this.on = value;
      },
    },
    querySelector() {
      return label;
    },
  };
  markLastCopied(
    {
      querySelectorAll() {
        return [row];
      },
    },
    "d",
  );
  assert.equal(row.classList.on, true);
  assert.equal(label.hidden, false);
});
