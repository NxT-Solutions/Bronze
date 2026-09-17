import assert from "node:assert/strict";
import { test } from "node:test";
import {
  animateElement,
  applyActionStatus,
  applyActionTip,
  bindOverflowDismiss,
  closeOverflowMenus,
  MOTION,
  motionAllowed,
  runBusy,
} from "./control.mjs";

test("runBusy sets aria-busy and restores the control", async () => {
  const el = { disabled: false, attrs: {} };
  el.getAttribute = (name) => el.attrs[name] ?? null;
  el.setAttribute = (name, value) => {
    el.attrs[name] = String(value);
  };
  el.removeAttribute = (name) => {
    delete el.attrs[name];
  };
  const result = await runBusy(el, async () => {
    assert.equal(el.getAttribute("aria-busy"), "true");
    assert.equal(el.disabled, true);
    return "ok";
  });
  assert.equal(result, "ok");
  assert.equal(el.getAttribute("aria-busy"), null);
  assert.equal(el.disabled, false);
});

test("action status copies catalog text and hides when missing", () => {
  const status = { textContent: "", hidden: true };
  const root = {
    querySelector(sel) {
      if (sel === "#action-status") {
        return status;
      }
      if (sel.includes("copy.announce.copied")) {
        return { textContent: "Copied to the clipboard." };
      }
      return null;
    },
  };
  applyActionStatus(root, "copy.announce.copied");
  assert.equal(status.textContent, "Copied to the clipboard.");
  assert.equal(status.hidden, false);
  applyActionStatus(root, "missing");
  assert.equal(status.textContent, "");
  assert.equal(status.hidden, true);
});

test("action tip writes catalog text on the row and overflow closes away", () => {
  const tip = { textContent: "", hidden: true, dataset: {} };
  const host = {
    querySelector(sel) {
      return sel.includes("action-tip") ? tip : null;
    },
  };
  const button = {
    closest(sel) {
      return sel === ".row-actions" ? host : null;
    },
    ownerDocument: { defaultView: null },
  };
  applyActionTip(button, "Copied to the clipboard.");
  assert.equal(tip.textContent, "Copied to the clipboard.");
  assert.equal(tip.hidden, false);
  applyActionStatus(
    {
      querySelector(sel) {
        if (sel === "#action-status") {
          return { textContent: "", hidden: true };
        }
        if (sel.includes("copy.announce.failed")) {
          return { textContent: "Copy did not write to the clipboard." };
        }
        return null;
      },
    },
    "copy.announce.failed",
    button,
  );
  assert.equal(tip.dataset.tone, "failed");

  const kept = { open: true };
  const other = { open: true };
  closeOverflowMenus(
    {
      querySelectorAll() {
        return [kept, other];
      },
    },
    kept,
  );
  assert.equal(kept.open, true);
  assert.equal(other.open, false);

  const listeners = [];
  const first = { open: true };
  const doc = {
    addEventListener(type, fn) {
      listeners.push({ type, fn });
    },
    querySelectorAll() {
      return [first];
    },
  };
  bindOverflowDismiss(doc);
  listeners
    .find((row) => row.type === "pointerdown")
    .fn({ target: { closest: () => null } });
  assert.equal(first.open, false);
});

test("motion helper stays local and yields under reduce-motion", () => {
  assert.equal(MOTION.duration, 180);
  assert.match(MOTION.easing, /cubic-bezier/);
  assert.equal(
    motionAllowed({
      documentElement: { hasAttribute: () => true },
    }),
    false,
  );
  assert.equal(
    motionAllowed({
      documentElement: { hasAttribute: () => false },
      defaultView: { matchMedia: () => ({ matches: true }) },
    }),
    false,
  );
  const played = [];
  const el = {
    ownerDocument: {
      documentElement: { hasAttribute: () => false },
      defaultView: { matchMedia: () => ({ matches: false }) },
    },
    animate(frames, options) {
      played.push({ frames, options });
      return { finished: Promise.resolve() };
    },
  };
  animateElement(el, [{ opacity: 0 }, { opacity: 1 }]);
  assert.equal(played.length, 1);
  assert.equal(played[0].options.duration, 180);
});
