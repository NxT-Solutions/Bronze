import assert from "node:assert/strict";
import { test } from "node:test";
import {
  animateElement,
  applyActionStatus,
  applyActionTip,
  bindChromeNotice,
  bindOverflowDismiss,
  closeOverflowMenus,
  hideChromeNotice,
  MOTION,
  motionAllowed,
  openEditSheet,
  runBusy,
  showChromeNotice,
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
        return { textContent: "Copied." };
      }
      return null;
    },
  };
  applyActionStatus(root, "copy.announce.copied");
  assert.equal(status.textContent, "Copied.");
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
  applyActionTip(button, "Copied.");
  assert.equal(tip.textContent, "Copied.");
  assert.equal(tip.hidden, false);
  applyActionStatus(
    {
      querySelector(sel) {
        if (sel === "#action-status") {
          return { textContent: "", hidden: true };
        }
        if (sel.includes("copy.announce.failed")) {
          return { textContent: "Could not copy." };
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

  const focused = [];
  const summary = { focus: () => focused.push("summary") };
  const openMenu = {
    open: true,
    querySelector() {
      return summary;
    },
  };
  const escapeRoot = {
    listeners: [],
    addEventListener(type, fn) {
      this.listeners.push({ type, fn });
    },
    querySelector() {
      return openMenu;
    },
    querySelectorAll() {
      return [openMenu];
    },
  };
  bindOverflowDismiss(escapeRoot);
  escapeRoot.listeners
    .find((row) => row.type === "keydown")
    .fn({
      key: "Escape",
      target: { closest: () => openMenu },
    });
  assert.equal(openMenu.open, false);
  assert.deepEqual(focused, ["summary"]);
});

test("chrome notice is viewport chrome and dismisses", () => {
  const label = { textContent: "" };
  const host = {
    hidden: true,
    dataset: {},
    querySelector(sel) {
      return sel === "#chrome-notice-text" ? label : null;
    },
    ownerDocument: { defaultView: null },
  };
  const root = {
    querySelector(sel) {
      return sel === "#chrome-notice" ? host : null;
    },
  };
  showChromeNotice(root, "This app is excluded.", "failed");
  assert.equal(label.textContent, "This app is excluded.");
  assert.equal(host.hidden, false);
  assert.equal(host.dataset.tone, "failed");
  hideChromeNotice(root);
  assert.equal(host.hidden, true);
  assert.equal(label.textContent, "");

  const clicks = [];
  const keys = [];
  const bound = {
    addEventListener(type, fn) {
      if (type === "click") {
        clicks.push(fn);
      }
      if (type === "keydown") {
        keys.push(fn);
      }
    },
    querySelector(sel) {
      return sel === "#chrome-notice" ? host : null;
    },
  };
  bindChromeNotice(bound);
  showChromeNotice(bound, "Could not add.", "failed");
  clicks[0]({
    target: {
      closest: (sel) => (sel === "[data-notice-dismiss]" ? true : null),
    },
  });
  assert.equal(host.hidden, true);
  showChromeNotice(bound, "Saved.");
  keys[0]({ key: "Escape" });
  assert.equal(host.hidden, true);
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

test("edit sheet saves or cancels without window.prompt", async () => {
  let closer;
  const field = { value: "", focus() {} };
  const dialog = {
    returnValue: "save",
    showModal() {},
    addEventListener(type, fn) {
      if (type === "close") {
        closer = fn;
      }
    },
    removeEventListener() {},
    querySelector() {
      return null;
    },
    close(value) {
      this.returnValue = value;
      closer?.();
    },
  };
  const root = {
    querySelector(sel) {
      if (sel === "#edit-sheet") {
        return dialog;
      }
      if (sel === "#edit-body") {
        return field;
      }
      return null;
    },
  };
  const leftover = openEditSheet(root, "stale");
  assert.equal(dialog.returnValue, "");
  closer();
  assert.equal(await leftover, null);
  const pending = openEditSheet(root, "hello");
  assert.equal(field.value, "hello");
  field.value = "world";
  dialog.returnValue = "save";
  closer();
  assert.equal(await pending, "world");
  const canceled = openEditSheet(root, "keep");
  dialog.returnValue = "cancel";
  closer();
  assert.equal(await canceled, null);
});
