import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  bindShortcutRegistry,
  chordFromKeyboardEvent,
  doubleTapFromModifierEvents,
  formatRecordingPreview,
  formatShortcutChord,
  isGlobalShortcutAction,
  modifierNameFromEvent,
  recorderIgnores,
  shortcutFailureKey,
} from "./shortcuts.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "settings.html"), "utf8");
const css = readFileSync(join(root, "chrome.css"), "utf8");
const live = readFileSync(join(root, "settings-live.mjs"), "utf8");
const registry = readFileSync(join(root, "shortcuts.mjs"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

const actions = [
  "app.togglePanel",
  "capture.selection",
  "capture.newNote",
  "queue.copy",
  "queue.copyWithProfile",
  "queue.copyAndAdvance",
  "queue.complete",
  "queue.edit",
  "queue.moveUp",
  "queue.moveDown",
  "queue.search",
  "queue.undo",
  "window.settings",
];

test("shortcut recorder lists every action and keeps capture.selection as standardChord", () => {
  assert.match(html, /data-slot="shortcut-recorder"/);
  assert.match(html, /data-standard-chord="capture.selection"/);
  for (const action of actions) {
    assert.match(html, new RegExp(`data-action="${action}"`));
    assert.match(
      html,
      new RegExp(`data-i18n="settings.shortcuts.action.${action}"`),
    );
  }
  assert.match(html, /data-shortcut-record/);
  assert.match(html, /data-shortcut-restore/);
  assert.doesNotMatch(html, /data-shortcut-record-bar/);
  assert.doesNotMatch(html, /id="shortcut-record"/);
  assert.match(html, /data-i18n="settings.shortcuts.restore"/);
  assert.doesNotMatch(html, />app\.togglePanel</);
  assert.match(html, /data-i18n="settings.shortcuts.action.app.togglePanel"/);
  assert.doesNotMatch(html, /data-i18n="settings.shortcuts.unassigned"/);
  assert.equal(en["settings.shortcuts.record"], "Record shortcut");
  assert.equal(en["settings.shortcuts.action.queue.copy"], "Copy");
  assert.equal(
    en["settings.shortcuts.live"],
    "Record a row to replace that action’s default.",
  );
  assert.match(html, /⌥Space/);
  assert.match(html, /⌘C/);
  assert.match(html, /shortcut-assign/);
  assert.match(html, /id="settings-shortcuts-record"/);
  assert.equal(
    html.match(/data-i18n="settings.shortcuts.live"[^>]*>\s*([^<]+?)\s*</)?.[1],
    en["settings.shortcuts.live"],
  );
  assert.equal(
    html.match(
      /data-i18n="settings.shortcuts.record"[^>]*>\s*([^<]+?)\s*</,
    )?.[1],
    en["settings.shortcuts.record"],
  );
  assert.equal(
    html.match(
      /data-i18n="settings.shortcuts.recording"[^>]*>\s*([^<]+?)\s*</,
    )?.[1],
    en["settings.shortcuts.recording"],
  );
  assert.equal(
    html.match(
      /data-i18n="settings.shortcuts.restore"[^>]*>\s*([^<]+?)\s*</,
    )?.[1],
    en["settings.shortcuts.restore"],
  );
  assert.equal(en["settings.shortcuts.recordingLabel"], "Recording…");
  assert.equal(
    html.match(
      /data-i18n="settings.shortcuts.recordingLabel"[^>]*>\s*([^<]+?)\s*</,
    )?.[1],
    en["settings.shortcuts.recordingLabel"],
  );
  assert.match(live, /bindShortcutRegistry/);
  assert.match(registry, /list_shortcuts/);
  assert.match(registry, /record_shortcut/);
  assert.match(registry, /restore_shortcut/);
  assert.match(css, /\.shortcut-list li \{[^}]*flex-wrap: wrap;/);
  assert.match(css, /\.shortcut-row-meta \{[^}]*flex-wrap: wrap;/);
});

test("shortcut formatter and recorder keep IME out and reject path keys", () => {
  assert.equal(isGlobalShortcutAction("app.togglePanel"), true);
  assert.equal(isGlobalShortcutAction("queue.copy"), false);
  assert.equal(
    formatShortcutChord({
      enabled: true,
      trigger: "accelerator",
      modifiers: ["Option"],
      logicalKey: " ",
    }),
    "⌥Space",
  );
  assert.equal(
    formatShortcutChord({
      enabled: true,
      trigger: "modifier_double_tap",
    }),
    "Shift double-tap",
  );
  assert.equal(
    formatShortcutChord({
      enabled: true,
      trigger: "modifier_double_tap",
      modifiers: ["Option"],
    }),
    "Option double-tap",
  );
  assert.equal(modifierNameFromEvent({ key: "Shift" }), "Shift");
  assert.equal(
    doubleTapFromModifierEvents(null, { key: "Shift" }, 100).previous.modifier,
    "Shift",
  );
  assert.deepEqual(
    doubleTapFromModifierEvents(
      { modifier: "Shift", at: 100 },
      { key: "Shift" },
      300,
    ).chord,
    {
      trigger: "modifier_double_tap",
      modifiers: ["Shift"],
      logicalKey: null,
      tapCount: 2,
    },
  );
  assert.equal(
    doubleTapFromModifierEvents(
      { modifier: "Shift", at: 100 },
      { key: "Shift" },
      700,
    ).chord,
    null,
  );
  assert.equal(
    doubleTapFromModifierEvents(
      { modifier: "Option", at: 100 },
      { key: "Dead", code: "" },
      180,
    ).previous.modifier,
    "Option",
  );
  assert.equal(formatRecordingPreview({ modifiers: ["Option"], taps: 1 }), "⌥");
  assert.equal(
    formatRecordingPreview({ modifiers: ["Option"], taps: 2 }),
    "Option double-tap",
  );
  assert.equal(
    formatRecordingPreview({ modifiers: ["Option"], taps: 3 }),
    "Option triple-tap",
  );
  assert.equal(
    formatRecordingPreview({ modifiers: ["Option"], taps: 4 }),
    "Option 4-tap",
  );
  assert.equal(
    formatShortcutChord({
      enabled: true,
      trigger: "modifier_double_tap",
      modifiers: ["Option"],
      tapCount: 3,
    }),
    "Option triple-tap",
  );
  assert.equal(
    formatRecordingPreview({
      modifiers: ["Option", "Shift"],
      logicalKey: "o",
      taps: 1,
    }),
    "⌥⇧O",
  );
  assert.equal(recorderIgnores({ isComposing: true, repeat: false }), true);
  assert.equal(
    recorderIgnores({ isComposing: true, key: "Alt", repeat: false }),
    false,
  );
  assert.equal(
    recorderIgnores({
      isComposing: true,
      key: "ø",
      code: "KeyO",
      altKey: true,
      shiftKey: true,
      repeat: false,
    }),
    false,
  );
  assert.deepEqual(
    doubleTapFromModifierEvents(
      { modifier: "Option", at: 100 },
      { key: "Alt", code: "AltLeft", isComposing: true },
      280,
    ).chord,
    {
      trigger: "modifier_double_tap",
      modifiers: ["Option"],
      logicalKey: null,
      tapCount: 2,
    },
  );
  assert.deepEqual(
    doubleTapFromModifierEvents(
      { modifier: "Option", at: 100, taps: 2 },
      { key: "Alt", code: "AltLeft", isComposing: true },
      360,
    ).chord,
    {
      trigger: "modifier_double_tap",
      modifiers: ["Option"],
      logicalKey: null,
      tapCount: 3,
    },
  );
  assert.deepEqual(
    chordFromKeyboardEvent({
      key: "k",
      metaKey: true,
      altKey: false,
      ctrlKey: false,
      shiftKey: false,
      isComposing: false,
      repeat: false,
    }),
    { trigger: "accelerator", modifiers: ["Command"], logicalKey: "k" },
  );
  assert.deepEqual(
    chordFromKeyboardEvent({
      key: "ø",
      code: "KeyO",
      metaKey: false,
      altKey: true,
      ctrlKey: false,
      shiftKey: true,
      isComposing: false,
      repeat: false,
    }),
    {
      trigger: "accelerator",
      modifiers: ["Option", "Shift"],
      logicalKey: "o",
    },
  );
  assert.equal(
    formatShortcutChord({
      enabled: true,
      trigger: "accelerator",
      modifiers: ["Option", "Shift"],
      logicalKey: "o",
    }),
    "⌥⇧O",
  );
  assert.equal(
    chordFromKeyboardEvent({
      key: "Backspace",
      code: "Backspace",
      isComposing: false,
      repeat: false,
    }),
    null,
  );
  assert.equal(
    chordFromKeyboardEvent({
      key: "Escape",
      isComposing: false,
      repeat: false,
    }).cancel,
    true,
  );
  assert.equal(
    shortcutFailureKey("shortcut_duplicate"),
    "settings.shortcuts.duplicate",
  );
  assert.equal(
    shortcutFailureKey("shortcut_rejected"),
    "settings.shortcuts.rejected",
  );
});

function fakeEl(attrs = {}, kids = []) {
  const listeners = {};
  const node = {
    attrs: { ...attrs },
    children: kids,
    parent: null,
    hidden: Boolean(attrs.hidden),
    value: "",
    textContent: attrs.textContent ?? "",
    disabled: false,
    dataset: {},
    id: attrs.id ?? "",
    getAttribute(name) {
      if (name in this.attrs) {
        return this.attrs[name];
      }
      return null;
    },
    setAttribute(name, value) {
      this.attrs[name] = String(value);
    },
    removeAttribute(name) {
      delete this.attrs[name];
    },
    addEventListener(type, fn) {
      if (!listeners[type]) {
        listeners[type] = [];
      }
      listeners[type].push(fn);
    },
    focus() {},
    contains(other) {
      return (
        this === other || this.children.some((child) => child.contains(other))
      );
    },
    closest(sel) {
      let current = this;
      while (current) {
        if (matchSel(current, sel)) {
          return current;
        }
        current = current.parent;
      }
      return null;
    },
    querySelector(sel) {
      return this.querySelectorAll(sel)[0] ?? null;
    },
    querySelectorAll(sel) {
      return collect(this, sel);
    },
    emit(type, event) {
      for (const fn of listeners[type] ?? []) {
        fn(event);
      }
    },
  };
  for (const child of kids) {
    child.parent = node;
  }
  return node;
}

function matchAttr(sel) {
  const raw = sel.slice(1, -1);
  const eq = raw.indexOf("=");
  if (eq === -1) {
    return (node) => node.getAttribute(raw) !== null || raw in node.attrs;
  }
  const key = raw.slice(0, eq);
  const value = raw.slice(eq + 1).replaceAll(/['"]/g, "");
  return (node) => node.getAttribute(key) === value;
}

function matchSel(node, sel) {
  if (sel.startsWith("#")) {
    return node.id === sel.slice(1);
  }
  if (sel.startsWith("[")) {
    return matchAttr(sel)(node);
  }
  return false;
}

function walk(node) {
  return node.children.flatMap((child) => [child, ...walk(child)]);
}

function collect(root, sel) {
  const parts = sel.split(/\s+/);
  let nodes = walk(root);
  if (root.parent === null) {
    nodes = [root, ...nodes];
  }
  if (parts.length === 1) {
    return nodes.filter((node) => matchSel(node, parts[0]));
  }
  const hosts = nodes.filter((node) => matchSel(node, parts[0]));
  return hosts.flatMap((host) =>
    walk(host).filter((node) => matchSel(node, parts[1])),
  );
}

function createClock() {
  const pending = new Map();
  let nextId = 1;
  return {
    setTimeout(fn) {
      const id = nextId++;
      pending.set(id, fn);
      return id;
    },
    clearTimeout(id) {
      pending.delete(id);
    },
    flush() {
      const fns = [...pending.values()];
      pending.clear();
      for (const fn of fns) {
        fn();
      }
    },
  };
}

test("shortcut registry paints defaults, records a custom chord, and restores", async () => {
  const liveStatus = fakeEl({ "data-shortcut-live": "" });
  const chord = fakeEl({ "data-slot": "shortcut-chord", textContent: "⌘F" });
  const recordBtn = fakeEl({ "data-shortcut-record": "" }, [chord]);
  const restore = fakeEl({ "data-shortcut-restore": "" });
  restore.hidden = true;
  const item = fakeEl({ "data-action": "queue.search" }, [recordBtn, restore]);
  const list = fakeEl({ "data-shortcut-registry": "" }, [item]);
  const root = fakeEl({}, [liveStatus, list]);
  list.ownerDocument = root;
  const clock = createClock();

  const calls = [];
  let rejectNext = false;
  const rows = [
    {
      action: "queue.search",
      trigger: "accelerator",
      modifiers: ["Command"],
      logicalKey: "f",
      enabled: true,
      isDefault: true,
    },
  ];
  const invokeFn = async (cmd, args) => {
    calls.push({ cmd, args });
    if (cmd === "record_shortcut" && rejectNext) {
      rejectNext = false;
      throw new Error("shortcut_rejected");
    }
    if (cmd === "record_shortcut") {
      rows[0] = {
        ...rows[0],
        trigger: args.input.trigger,
        logicalKey: args.input.logicalKey,
        modifiers: args.input.modifiers,
        tapCount: args.input.tapCount,
        isDefault: false,
      };
    }
    if (cmd === "restore_shortcut") {
      rows[0] = {
        ...rows[0],
        trigger: "accelerator",
        logicalKey: "f",
        modifiers: ["Command"],
        tapCount: undefined,
        isDefault: true,
      };
    }
    return rows;
  };

  await bindShortcutRegistry(root, invokeFn, clock);
  assert.equal(chord.textContent, "⌘F");
  assert.equal(restore.hidden, true);

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  assert.equal(list.dataset.recordingAction, "queue.search");
  assert.equal(item.dataset.recording, "true");
  assert.equal(recordBtn.getAttribute("aria-pressed"), "true");
  assert.equal(chord.textContent, "Recording…");
  assert.equal(
    liveStatus.textContent,
    "Type a shortcut or tap a modifier two or more times (Escape cancels).",
  );
  assert.equal(liveStatus.dataset.recording, "true");

  root.emit("keydown", {
    key: "Escape",
    code: "Escape",
    isComposing: false,
    repeat: false,
    preventDefault() {},
  });
  assert.equal(list.dataset.recordingAction, undefined);
  assert.equal(item.dataset.recording, undefined);
  assert.equal(recordBtn.getAttribute("aria-pressed"), "false");
  assert.equal(chord.textContent, "⌘F");
  assert.equal(restore.hidden, true);
  assert.equal(liveStatus.dataset.recording, undefined);

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  assert.equal(chord.textContent, "Recording…");

  root.emit("keydown", {
    key: "Alt",
    code: "AltLeft",
    altKey: true,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  assert.equal(chord.textContent, "⌥");
  root.emit("keydown", {
    key: "Shift",
    code: "ShiftLeft",
    altKey: true,
    shiftKey: true,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  assert.equal(chord.textContent, "⌥⇧");
  root.emit("keydown", {
    key: "ø",
    code: "KeyO",
    metaKey: false,
    altKey: true,
    ctrlKey: false,
    shiftKey: true,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  await Promise.resolve();
  assert.equal(calls.at(-1).cmd, "record_shortcut");
  assert.deepEqual(calls.at(-1).args.input, {
    action: "queue.search",
    trigger: "accelerator",
    modifiers: ["Option", "Shift"],
    logicalKey: "o",
    skipTest: true,
  });
  assert.equal(chord.textContent, "⌥⇧O");
  assert.equal(restore.hidden, false);
  assert.equal(list.dataset.recordingAction, undefined);

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  assert.equal(chord.textContent, "Recording…");
  assert.equal(restore.hidden, false);
  root.emit("keydown", {
    key: "Escape",
    code: "Escape",
    isComposing: false,
    repeat: false,
    preventDefault() {},
  });
  assert.equal(chord.textContent, "⌥⇧O");
  assert.equal(restore.hidden, false);

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return null;
        if (sel === "[data-shortcut-restore]") return restore;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(calls.at(-1).cmd, "restore_shortcut");
  assert.equal(calls.at(-1).args.action, "queue.search");
  assert.equal(chord.textContent, "⌘F");
  assert.equal(restore.hidden, true);

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  root.emit("keydown", {
    key: "Meta",
    code: "MetaLeft",
    metaKey: true,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  root.emit("keydown", {
    key: "c",
    code: "KeyC",
    metaKey: true,
    altKey: false,
    ctrlKey: false,
    shiftKey: false,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  await Promise.resolve();
  assert.equal(calls.at(-1).cmd, "record_shortcut");
  assert.deepEqual(calls.at(-1).args.input, {
    action: "queue.search",
    trigger: "accelerator",
    modifiers: ["Command"],
    logicalKey: "c",
    skipTest: true,
  });
  assert.equal(chord.textContent, "⌘C");
  root.emit("keyup", {
    key: "Meta",
    code: "MetaLeft",
    timeStamp: 40,
    isComposing: false,
    repeat: false,
    preventDefault() {},
  });
  assert.equal(calls.at(-1).cmd, "record_shortcut");
  assert.equal(calls.at(-1).args.input.trigger, "accelerator");

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  const beforeDoubleTap = calls.length;
  root.emit("keydown", {
    key: "Shift",
    code: "ShiftLeft",
    timeStamp: 100,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  assert.equal(calls.length, beforeDoubleTap);
  assert.equal(chord.textContent, "⇧");
  root.emit("keyup", {
    key: "Shift",
    code: "ShiftLeft",
    timeStamp: 160,
    isComposing: false,
    repeat: false,
    preventDefault() {},
  });
  assert.equal(calls.length, beforeDoubleTap);
  root.emit("keydown", {
    key: "Shift",
    code: "ShiftLeft",
    timeStamp: 280,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  await Promise.resolve();
  assert.equal(calls.length, beforeDoubleTap);
  assert.equal(chord.textContent, "Shift double-tap");
  clock.flush();
  await Promise.resolve();
  assert.equal(calls.at(-1).cmd, "record_shortcut");
  assert.deepEqual(calls.at(-1).args.input, {
    action: "queue.search",
    trigger: "modifier_double_tap",
    modifiers: ["Shift"],
    logicalKey: null,
    tapCount: 2,
    skipTest: true,
  });
  assert.equal(chord.textContent, "Shift double-tap");
  assert.equal(restore.hidden, false);

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  const optionTap = (timeStamp, type) => ({
    key: "Alt",
    code: "AltLeft",
    timeStamp,
    altKey: type === "keydown",
    isComposing: true,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  const beforeOptionDouble = calls.length;
  root.emit("keydown", optionTap(400, "keydown"));
  assert.equal(chord.textContent, "⌥");
  root.emit("keyup", optionTap(460, "keyup"));
  root.emit("keydown", optionTap(520, "keydown"));
  await Promise.resolve();
  assert.equal(calls.length, beforeOptionDouble);
  assert.equal(chord.textContent, "Option double-tap");
  clock.flush();
  await Promise.resolve();
  assert.equal(calls.at(-1).cmd, "record_shortcut");
  assert.deepEqual(calls.at(-1).args.input, {
    action: "queue.search",
    trigger: "modifier_double_tap",
    modifiers: ["Option"],
    logicalKey: null,
    tapCount: 2,
    skipTest: true,
  });
  assert.equal(chord.textContent, "Option double-tap");

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  const beforeDeadDouble = calls.length;
  root.emit("keydown", optionTap(800, "keydown"));
  assert.equal(chord.textContent, "⌥");
  root.emit("keyup", {
    key: "Dead",
    code: "",
    timeStamp: 820,
    isComposing: true,
    repeat: false,
    preventDefault() {},
  });
  assert.equal(chord.textContent, "⌥");
  root.emit("keydown", optionTap(860, "keydown"));
  await Promise.resolve();
  assert.equal(calls.length, beforeDeadDouble);
  assert.equal(chord.textContent, "Option double-tap");
  clock.flush();
  await Promise.resolve();
  assert.equal(calls.at(-1).args.input.trigger, "modifier_double_tap");
  assert.equal(calls.at(-1).args.input.modifiers[0], "Option");
  assert.equal(calls.at(-1).args.input.tapCount, 2);

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  const beforeTriple = calls.length;
  root.emit("keydown", optionTap(1000, "keydown"));
  assert.equal(chord.textContent, "⌥");
  root.emit("keydown", optionTap(1080, "keydown"));
  assert.equal(chord.textContent, "Option double-tap");
  assert.equal(calls.length, beforeTriple);
  root.emit("keydown", optionTap(1160, "keydown"));
  await Promise.resolve();
  assert.equal(calls.length, beforeTriple);
  assert.equal(chord.textContent, "Option triple-tap");
  clock.flush();
  await Promise.resolve();
  assert.equal(calls.at(-1).args.input.tapCount, 3);
  assert.equal(chord.textContent, "Option triple-tap");

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  const beforeSingleTap = calls.length;
  root.emit("keydown", optionTap(1400, "keydown"));
  assert.equal(chord.textContent, "⌥");
  clock.flush();
  await Promise.resolve();
  assert.equal(calls.length, beforeSingleTap);
  assert.equal(list.dataset.recordingAction, "queue.search");

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  rejectNext = true;
  root.emit("keydown", {
    key: "Meta",
    code: "MetaLeft",
    metaKey: true,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  root.emit("keydown", {
    key: "c",
    code: "KeyC",
    metaKey: true,
    altKey: false,
    ctrlKey: false,
    shiftKey: false,
    isComposing: false,
    repeat: false,
    preventDefault() {},
    stopPropagation() {},
  });
  await Promise.resolve();
  await Promise.resolve();
  const rejected = calls.findLast((entry) => entry.cmd === "record_shortcut");
  assert.deepEqual(rejected.args.input, {
    action: "queue.search",
    trigger: "accelerator",
    modifiers: ["Command"],
    logicalKey: "c",
    skipTest: true,
  });
  assert.equal(list.dataset.recordingAction, "queue.search");
  assert.equal(liveStatus.textContent, "Could not save that shortcut.");
  assert.equal(chord.textContent, "⌘C");
  const recordsAfterReject = calls.filter(
    (entry) => entry.cmd === "record_shortcut",
  ).length;
  root.emit("keyup", {
    key: "Meta",
    code: "MetaLeft",
    timeStamp: 700,
    isComposing: false,
    repeat: false,
    preventDefault() {},
  });
  assert.equal(
    calls.filter((entry) => entry.cmd === "record_shortcut").length,
    recordsAfterReject,
  );
  assert.equal(list.dataset.recordingAction, "queue.search");

  list.emit("click", {
    target: {
      closest(sel) {
        if (sel === "[data-shortcut-record]") return recordBtn;
        if (sel === "[data-shortcut-restore]") return null;
        if (sel === "[data-action]") return item;
        return null;
      },
    },
  });
  rejectNext = false;
  root.emit("copy", {
    preventDefault() {},
  });
  await Promise.resolve();
  const copied = calls.findLast((entry) => entry.cmd === "record_shortcut");
  assert.deepEqual(copied.args.input, {
    action: "queue.search",
    trigger: "accelerator",
    modifiers: ["Command"],
    logicalKey: "c",
    skipTest: true,
  });
});
