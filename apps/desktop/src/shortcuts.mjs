import { catalogMessage } from "./apply-locale.mjs";
import { animateElement, runBusy } from "./control.mjs";

export const SHORTCUT_ACTIONS = [
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

const MODIFIER_GLYPH = {
  Command: "⌘",
  Option: "⌥",
  Control: "⌃",
  Shift: "⇧",
  Fn: "Fn",
};

export function isGlobalShortcutAction(action) {
  return action === "app.togglePanel" || action === "capture.selection";
}

const CODE_KEYS = {
  Space: " ",
  Enter: "Enter",
  NumpadEnter: "Enter",
  Escape: "Escape",
  Tab: "Tab",
  ArrowUp: "ArrowUp",
  ArrowDown: "ArrowDown",
  ArrowLeft: "ArrowLeft",
  ArrowRight: "ArrowRight",
  Backspace: "Backspace",
  Delete: "Delete",
  Comma: ",",
  Period: ".",
  Semicolon: ";",
  Quote: "'",
  BracketLeft: "[",
  BracketRight: "]",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
};

export function normalizeLogicalKey(key) {
  if (key === " ") {
    return " ";
  }
  if (key.length === 1 && /[A-Za-z0-9,.;'[\]`=-]/.test(key)) {
    return key.toLowerCase();
  }
  if (
    [
      "Enter",
      "Escape",
      "Tab",
      "ArrowUp",
      "ArrowDown",
      "ArrowLeft",
      "ArrowRight",
      "Backspace",
      "Delete",
    ].includes(key)
  ) {
    return key;
  }
  if (/^F([1-9]|1[0-9])$/.test(key)) {
    return key;
  }
  return "";
}

export function logicalKeyFromCode(code) {
  if (!code) {
    return "";
  }
  if (/^Key[A-Z]$/.test(code)) {
    return code.slice(3).toLowerCase();
  }
  if (/^Digit[0-9]$/.test(code)) {
    return code.slice(5);
  }
  if (CODE_KEYS[code]) {
    return CODE_KEYS[code];
  }
  if (/^F([1-9]|1[0-9])$/.test(code)) {
    return code;
  }
  return "";
}

export function logicalKeyFromEvent(event) {
  const fromKey = normalizeLogicalKey(event?.key ?? "");
  if (fromKey && !event?.altKey) {
    return fromKey;
  }
  // Option remaps letters to symbols (ø); physical code keeps the letter.
  return logicalKeyFromCode(event?.code) || fromKey;
}

export const MODIFIER_DOUBLE_TAP_GAP_MS = 500;
export const MAX_MODIFIER_TAPS = 8;

const KEY_MODIFIERS = {
  Meta: "Command",
  Alt: "Option",
  AltGraph: "Option",
  Option: "Option",
  Control: "Control",
  Shift: "Shift",
  Fn: "Fn",
};

const CODE_MODIFIERS = {
  MetaLeft: "Command",
  MetaRight: "Command",
  AltLeft: "Option",
  AltRight: "Option",
  ControlLeft: "Control",
  ControlRight: "Control",
  ShiftLeft: "Shift",
  ShiftRight: "Shift",
};

export function modifierNameFromEvent(event) {
  if (!event) {
    return "";
  }
  return KEY_MODIFIERS[event.key] || CODE_MODIFIERS[event.code] || "";
}

function eventHasModifier(event) {
  return Boolean(
    event?.metaKey ||
      event?.altKey ||
      event?.ctrlKey ||
      event?.shiftKey ||
      modifierNameFromEvent(event),
  );
}

export function recorderIgnores(event) {
  return Boolean(
    (event?.isComposing && !eventHasModifier(event)) ||
      event?.repeat ||
      (event?.code === "F5" && event?.ctrlKey),
  );
}

export function chordFromKeyboardEvent(event) {
  if (recorderIgnores(event)) {
    return null;
  }
  if (event.key === "Escape" || event.code === "Escape") {
    return { cancel: true };
  }
  if (modifierNameFromEvent(event)) {
    return null;
  }
  const logicalKey = logicalKeyFromEvent(event);
  if (!logicalKey) {
    return null;
  }
  const modifiers = [];
  if (event.metaKey) {
    modifiers.push("Command");
  }
  if (event.altKey) {
    modifiers.push("Option");
  }
  if (event.ctrlKey) {
    modifiers.push("Control");
  }
  if (event.shiftKey) {
    modifiers.push("Shift");
  }
  if (
    modifiers.length === 0 &&
    (logicalKey === "Backspace" || logicalKey === "Delete")
  ) {
    return null;
  }
  return { trigger: "accelerator", modifiers, logicalKey };
}

export function formatShortcutKey(logicalKey, catalog = {}) {
  if (logicalKey === " ") {
    return catalog["settings.shortcuts.key.space"] || "Space";
  }
  if (logicalKey === "Enter") {
    return catalog["settings.shortcuts.key.enter"] || "Return";
  }
  if (logicalKey === "ArrowUp") {
    return catalog["settings.shortcuts.key.arrowUp"] || "↑";
  }
  if (logicalKey === "ArrowDown") {
    return catalog["settings.shortcuts.key.arrowDown"] || "↓";
  }
  if (!logicalKey) {
    return "";
  }
  return logicalKey.length === 1 ? logicalKey.toUpperCase() : logicalKey;
}

const DOUBLE_TAP_KEYS = {
  Shift: "settings.shortcuts.chord.shiftDoubleTap",
  Option: "settings.shortcuts.chord.optionDoubleTap",
  Command: "settings.shortcuts.chord.commandDoubleTap",
  Control: "settings.shortcuts.chord.controlDoubleTap",
  Fn: "settings.shortcuts.chord.fnDoubleTap",
};

const DOUBLE_TAP_FALLBACK = {
  Shift: "Shift double-tap",
  Option: "Option double-tap",
  Command: "Command double-tap",
  Control: "Control double-tap",
  Fn: "Fn double-tap",
};

export function doubleTapFromModifierEvents(previous, event, now) {
  if (recorderIgnores(event)) {
    return { previous, chord: null };
  }
  const modifier = modifierNameFromEvent(event);
  if (!modifier) {
    return { previous, chord: null };
  }
  if (
    previous &&
    previous.modifier === modifier &&
    now - previous.at <= MODIFIER_DOUBLE_TAP_GAP_MS
  ) {
    const taps = Math.min((previous.taps ?? 1) + 1, MAX_MODIFIER_TAPS);
    return {
      previous: { modifier, at: now, taps },
      chord: {
        trigger: "modifier_double_tap",
        modifiers: [modifier],
        logicalKey: null,
        tapCount: taps,
      },
    };
  }
  return { previous: { modifier, at: now, taps: 1 }, chord: null };
}

const TRIPLE_TAP_KEYS = {
  Shift: "settings.shortcuts.chord.shiftTripleTap",
  Option: "settings.shortcuts.chord.optionTripleTap",
  Command: "settings.shortcuts.chord.commandTripleTap",
  Control: "settings.shortcuts.chord.controlTripleTap",
  Fn: "settings.shortcuts.chord.fnTripleTap",
};

const TRIPLE_TAP_FALLBACK = {
  Shift: "Shift triple-tap",
  Option: "Option triple-tap",
  Command: "Command triple-tap",
  Control: "Control triple-tap",
  Fn: "Fn triple-tap",
};

const MODIFIER_NAME_KEYS = {
  Shift: "settings.shortcuts.modifier.shift",
  Option: "settings.shortcuts.modifier.option",
  Command: "settings.shortcuts.modifier.command",
  Control: "settings.shortcuts.modifier.control",
  Fn: "settings.shortcuts.modifier.fn",
};

function effectiveTapCount(tapCount) {
  const count = Number(tapCount);
  if (!Number.isFinite(count)) {
    return 2;
  }
  return Math.min(MAX_MODIFIER_TAPS, Math.max(2, count));
}

function formatCatalogTemplate(template, values) {
  let out = template;
  for (const [key, value] of Object.entries(values)) {
    out = out.replaceAll(`{${key}}`, String(value));
  }
  return out;
}

export function formatModifierDoubleTap(modifier, catalog = {}, tapCount = 2) {
  const count = effectiveTapCount(tapCount);
  if (count === 2) {
    const key = DOUBLE_TAP_KEYS[modifier];
    const fallback = DOUBLE_TAP_FALLBACK[modifier] || DOUBLE_TAP_FALLBACK.Shift;
    return (key && catalog[key]) || fallback;
  }
  if (count === 3) {
    const key = TRIPLE_TAP_KEYS[modifier];
    const fallback = TRIPLE_TAP_FALLBACK[modifier] || TRIPLE_TAP_FALLBACK.Shift;
    return (key && catalog[key]) || fallback;
  }
  const nameKey = MODIFIER_NAME_KEYS[modifier];
  const name = (nameKey && catalog[nameKey]) || modifier || "Shift";
  const template =
    catalog["settings.shortcuts.chord.modifierNTap"] ||
    "{modifier} {count}-tap";
  return formatCatalogTemplate(template, { modifier: name, count });
}

export function formatRecordingPreview(partial, labels = {}) {
  const modifiers = partial?.modifiers ?? [];
  const taps = partial?.taps ?? 0;
  if (taps >= 2 && modifiers.length === 1 && !partial?.logicalKey) {
    return formatModifierDoubleTap(modifiers[0], labels, taps);
  }
  const glyphs = modifiers
    .map((modifier) => MODIFIER_GLYPH[modifier] ?? "")
    .join("");
  return `${glyphs}${formatShortcutKey(partial?.logicalKey, labels)}`;
}

function modifiersFromEvent(event) {
  const current = modifierNameFromEvent(event);
  const modifiers = [];
  if (event?.metaKey || current === "Command") {
    modifiers.push("Command");
  }
  if (event?.altKey || current === "Option") {
    modifiers.push("Option");
  }
  if (event?.ctrlKey || current === "Control") {
    modifiers.push("Control");
  }
  if (event?.shiftKey || current === "Shift") {
    modifiers.push("Shift");
  }
  if (current === "Fn") {
    modifiers.push("Fn");
  }
  return modifiers;
}

export function formatShortcutChord(row, catalog = {}) {
  if (!row?.enabled || row.trigger === "disabled") {
    return catalog["settings.shortcuts.unassigned"] || "Not assigned";
  }
  if (row.trigger === "modifier_double_tap") {
    return formatModifierDoubleTap(row.modifiers?.[0], catalog, row.tapCount);
  }
  const glyphs = (row.modifiers ?? [])
    .map((modifier) => MODIFIER_GLYPH[modifier] ?? "")
    .join("");
  return `${glyphs}${formatShortcutKey(row.logicalKey, catalog)}`;
}

export function shortcutFailureKey(error) {
  const text =
    typeof error === "string" ? error : String(error?.message ?? error ?? "");
  if (text.includes("shortcut_duplicate")) {
    return "settings.shortcuts.duplicate";
  }
  return "settings.shortcuts.rejected";
}

const LIVE_FALLBACK = "Record a row to replace that action’s default.";
const RECORDING_FALLBACK =
  "Type a shortcut or tap a modifier two or more times (Escape cancels).";
const RECORDING_LABEL_FALLBACK = "Recording…";
const REJECTED_FALLBACK = "Could not save that shortcut.";

function message(key, fallback) {
  return catalogMessage(key) || fallback;
}

function catalog() {
  return {
    "settings.shortcuts.unassigned": message(
      "settings.shortcuts.unassigned",
      "Not assigned",
    ),
    "settings.shortcuts.chord.shiftDoubleTap": message(
      "settings.shortcuts.chord.shiftDoubleTap",
      "Shift double-tap",
    ),
    "settings.shortcuts.chord.optionDoubleTap": message(
      "settings.shortcuts.chord.optionDoubleTap",
      "Option double-tap",
    ),
    "settings.shortcuts.chord.commandDoubleTap": message(
      "settings.shortcuts.chord.commandDoubleTap",
      "Command double-tap",
    ),
    "settings.shortcuts.chord.controlDoubleTap": message(
      "settings.shortcuts.chord.controlDoubleTap",
      "Control double-tap",
    ),
    "settings.shortcuts.chord.fnDoubleTap": message(
      "settings.shortcuts.chord.fnDoubleTap",
      "Fn double-tap",
    ),
    "settings.shortcuts.chord.shiftTripleTap": message(
      "settings.shortcuts.chord.shiftTripleTap",
      "Shift triple-tap",
    ),
    "settings.shortcuts.chord.optionTripleTap": message(
      "settings.shortcuts.chord.optionTripleTap",
      "Option triple-tap",
    ),
    "settings.shortcuts.chord.commandTripleTap": message(
      "settings.shortcuts.chord.commandTripleTap",
      "Command triple-tap",
    ),
    "settings.shortcuts.chord.controlTripleTap": message(
      "settings.shortcuts.chord.controlTripleTap",
      "Control triple-tap",
    ),
    "settings.shortcuts.chord.fnTripleTap": message(
      "settings.shortcuts.chord.fnTripleTap",
      "Fn triple-tap",
    ),
    "settings.shortcuts.chord.modifierNTap": message(
      "settings.shortcuts.chord.modifierNTap",
      "{modifier} {count}-tap",
    ),
    "settings.shortcuts.modifier.shift": message(
      "settings.shortcuts.modifier.shift",
      "Shift",
    ),
    "settings.shortcuts.modifier.option": message(
      "settings.shortcuts.modifier.option",
      "Option",
    ),
    "settings.shortcuts.modifier.command": message(
      "settings.shortcuts.modifier.command",
      "Command",
    ),
    "settings.shortcuts.modifier.control": message(
      "settings.shortcuts.modifier.control",
      "Control",
    ),
    "settings.shortcuts.modifier.fn": message(
      "settings.shortcuts.modifier.fn",
      "Fn",
    ),
    "settings.shortcuts.key.space": message(
      "settings.shortcuts.key.space",
      "Space",
    ),
    "settings.shortcuts.key.enter": message(
      "settings.shortcuts.key.enter",
      "Return",
    ),
    "settings.shortcuts.key.arrowUp": message(
      "settings.shortcuts.key.arrowUp",
      "↑",
    ),
    "settings.shortcuts.key.arrowDown": message(
      "settings.shortcuts.key.arrowDown",
      "↓",
    ),
  };
}

function setLive(root, key, fallback) {
  const live = root.querySelector("[data-shortcut-live]");
  if (!live) {
    return;
  }
  live.textContent = message(key, fallback);
  if (key === "settings.shortcuts.recording") {
    live.dataset.recording = "true";
  } else {
    delete live.dataset.recording;
  }
}

function recordingLabel() {
  return message("settings.shortcuts.recordingLabel", RECORDING_LABEL_FALLBACK);
}

function beginRecordingChord(item) {
  const slot = item.querySelector("[data-slot='shortcut-chord']");
  if (!slot) {
    return;
  }
  if (item.dataset.idleChord == null) {
    item.dataset.idleChord = slot.textContent ?? "";
  }
  delete item.dataset.previewChord;
  slot.textContent = recordingLabel();
}

function endRecordingChord(item) {
  const slot = item.querySelector("[data-slot='shortcut-chord']");
  if (slot && item.dataset.idleChord != null) {
    slot.textContent = item.dataset.idleChord;
  }
  delete item.dataset.idleChord;
  delete item.dataset.previewChord;
}

function previewRecording(item, text) {
  const slot = item?.querySelector?.("[data-slot='shortcut-chord']");
  if (!item || !slot || !text) {
    return;
  }
  if (item.dataset.idleChord == null) {
    item.dataset.idleChord = slot.textContent ?? "";
  }
  const changed = slot.textContent !== text;
  item.dataset.previewChord = text;
  slot.textContent = text;
  if (!changed) {
    return;
  }
  const assign = item.querySelector("[data-shortcut-record]") ?? slot;
  animateElement(
    assign,
    [
      { transform: "scale(1)" },
      { transform: "scale(1.06)" },
      { transform: "scale(1)" },
    ],
    { duration: 180 },
  );
}

function paintRows(root, rows) {
  const labels = catalog();
  for (const row of rows ?? []) {
    const item = root.querySelector(
      `[data-shortcut-registry] [data-action="${row.action}"]`,
    );
    if (!item) {
      continue;
    }
    const chord = item.querySelector("[data-slot='shortcut-chord']");
    if (chord) {
      const formatted = formatShortcutChord(row, labels);
      if (item.dataset.recording) {
        item.dataset.idleChord = formatted;
        chord.textContent = item.dataset.previewChord || recordingLabel();
      } else {
        delete item.dataset.idleChord;
        chord.textContent = formatted;
      }
    }
    const restore = item.querySelector("[data-shortcut-restore]");
    if (restore) {
      restore.hidden = Boolean(row.isDefault);
    }
  }
}

function setRecordingRow(root, action) {
  for (const item of root.querySelectorAll(
    "[data-shortcut-registry] [data-action]",
  )) {
    const match =
      Boolean(action) && item.getAttribute("data-action") === action;
    if (match) {
      item.dataset.recording = "true";
      beginRecordingChord(item);
    } else {
      if (item.dataset.recording) {
        endRecordingChord(item);
      }
      delete item.dataset.recording;
    }
    item
      .querySelector("[data-shortcut-record]")
      ?.setAttribute("aria-pressed", match ? "true" : "false");
  }
}

function closeRecorder(root) {
  const list = root.querySelector("[data-shortcut-registry]");
  if (list) {
    delete list.dataset.recordingAction;
  }
  setRecordingRow(root, "");
}

export async function bindShortcutRegistry(root, invokeFn, clock = globalThis) {
  const list = root.querySelector("[data-shortcut-registry]");
  if (!list || !invokeFn) {
    return;
  }

  const refresh = async () => {
    try {
      const rows = await invokeFn("list_shortcuts");
      if (Array.isArray(rows)) {
        paintRows(root, rows);
      }
    } catch {
      setLive(root, "settings.shortcuts.rejected", REJECTED_FALLBACK);
    }
  };

  let modifierTap = null;
  let usedModifierWithKey = false;
  let countedPress = false;
  let pendingTimer = null;
  let pendingCommit = null;

  const recordingItem = (action) =>
    list.querySelector(`[data-action="${action}"]`);

  const showPreview = (action, partial) => {
    previewRecording(
      recordingItem(action),
      formatRecordingPreview(partial, catalog()),
    );
  };

  const clearPendingTap = () => {
    if (pendingTimer != null) {
      clock.clearTimeout(pendingTimer);
      pendingTimer = null;
    }
    pendingCommit = null;
  };

  const record = async (action, chord, skipTest) => {
    clearPendingTap();
    if (isGlobalShortcutAction(action) && chord.modifiers.length === 0) {
      setLive(root, "settings.shortcuts.rejected", REJECTED_FALLBACK);
      await refresh();
      return;
    }
    modifierTap = null;
    countedPress = false;
    delete list.dataset.recordingAction;
    const input = {
      action,
      trigger: chord.trigger,
      modifiers: chord.modifiers,
      logicalKey: chord.logicalKey,
      skipTest,
    };
    if (chord.trigger === "modifier_double_tap") {
      input.tapCount = effectiveTapCount(chord.tapCount);
    }
    try {
      const rows = await invokeFn("record_shortcut", { input });
      if (Array.isArray(rows)) {
        paintRows(root, rows);
      }
      closeRecorder(root);
      setLive(root, "settings.shortcuts.live", LIVE_FALLBACK);
    } catch (error) {
      list.dataset.recordingAction = action;
      setLive(root, shortcutFailureKey(error), REJECTED_FALLBACK);
      await refresh();
    }
  };

  const scheduleModifierTapCommit = (action, chord) => {
    pendingCommit = { action, chord };
    if (effectiveTapCount(chord.tapCount) >= MAX_MODIFIER_TAPS) {
      record(action, chord, true);
      return;
    }
    if (pendingTimer != null) {
      clock.clearTimeout(pendingTimer);
    }
    pendingTimer = clock.setTimeout(() => {
      pendingTimer = null;
      const next = pendingCommit;
      pendingCommit = null;
      if (next) {
        record(next.action, next.chord, true);
      }
    }, MODIFIER_DOUBLE_TAP_GAP_MS);
  };

  list.addEventListener("click", (event) => {
    const recordButton = event.target?.closest?.("[data-shortcut-record]");
    const restoreButton = event.target?.closest?.("[data-shortcut-restore]");
    const item = event.target?.closest?.("[data-action]");
    if (!item || !list.contains(item)) {
      return;
    }
    const action = item.getAttribute("data-action");
    if (restoreButton) {
      runBusy(restoreButton, async () => {
        try {
          clearPendingTap();
          const rows = await invokeFn("restore_shortcut", { action });
          if (Array.isArray(rows)) {
            paintRows(root, rows);
          }
          closeRecorder(root);
          setLive(root, "settings.shortcuts.live", LIVE_FALLBACK);
        } catch (error) {
          setLive(root, shortcutFailureKey(error), REJECTED_FALLBACK);
        }
      });
      return;
    }
    if (recordButton) {
      clearPendingTap();
      modifierTap = null;
      usedModifierWithKey = false;
      countedPress = false;
      list.dataset.recordingAction = action;
      setRecordingRow(root, action);
      recordButton.focus();
      setLive(root, "settings.shortcuts.recording", RECORDING_FALLBACK);
    }
  });

  const applyModifierTap = (event, action) => {
    const tapped = doubleTapFromModifierEvents(modifierTap, event, Date.now());
    modifierTap = tapped.previous;
    if (!tapped.chord) {
      clearPendingTap();
      if (tapped.previous) {
        showPreview(action, {
          modifiers: [tapped.previous.modifier],
          taps: tapped.previous.taps ?? 1,
        });
      }
      return;
    }
    showPreview(action, {
      modifiers: tapped.chord.modifiers,
      taps: tapped.chord.tapCount,
    });
    scheduleModifierTapCommit(action, tapped.chord);
  };

  const onKeyDown = (event) => {
    const action = list.dataset.recordingAction;
    if (!action) {
      return;
    }
    if (recorderIgnores(event)) {
      return;
    }
    const chord = chordFromKeyboardEvent(event);
    event.preventDefault();
    if (typeof event.stopPropagation === "function") {
      event.stopPropagation();
    }
    if (chord?.cancel) {
      clearPendingTap();
      modifierTap = null;
      usedModifierWithKey = false;
      countedPress = false;
      closeRecorder(root);
      setLive(root, "settings.shortcuts.live", LIVE_FALLBACK);
      return;
    }
    if (chord) {
      usedModifierWithKey = true;
      countedPress = false;
      modifierTap = null;
      clearPendingTap();
      showPreview(action, {
        modifiers: chord.modifiers,
        logicalKey: chord.logicalKey,
        taps: 1,
      });
      record(action, chord, true);
      return;
    }
    const modifier = modifierNameFromEvent(event);
    if (!modifier) {
      return;
    }
    const held = modifiersFromEvent(event);
    if (held.length > 1) {
      clearPendingTap();
      modifierTap = null;
      countedPress = true;
      showPreview(action, { modifiers: held, taps: 1 });
      return;
    }
    countedPress = true;
    applyModifierTap(event, action);
  };

  const onKeyUp = (event) => {
    const action = list.dataset.recordingAction;
    if (!action || recorderIgnores(event) || !modifierNameFromEvent(event)) {
      return;
    }
    event.preventDefault();
    if (usedModifierWithKey) {
      usedModifierWithKey = false;
      countedPress = false;
      return;
    }
    if (countedPress) {
      countedPress = false;
      return;
    }
    applyModifierTap(event, action);
  };

  const onCopy = (event) => {
    const action = list.dataset.recordingAction;
    if (!action) {
      return;
    }
    event.preventDefault();
    usedModifierWithKey = true;
    countedPress = false;
    modifierTap = null;
    clearPendingTap();
    showPreview(action, {
      modifiers: ["Command"],
      logicalKey: "c",
      taps: 1,
    });
    record(
      action,
      {
        trigger: "accelerator",
        modifiers: ["Command"],
        logicalKey: "c",
      },
      true,
    );
  };

  const host = list.ownerDocument ?? list;
  host.addEventListener("keydown", onKeyDown, true);
  host.addEventListener("keyup", onKeyUp, true);
  host.addEventListener("copy", onCopy, true);

  await refresh();
  return refresh;
}
