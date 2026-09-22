import { catalogMessage } from "./apply-locale.mjs";
import { runBusy } from "./control.mjs";

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

export function recorderIgnores(event) {
  return Boolean(
    event?.isComposing ||
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
  if (["Meta", "Alt", "Control", "Shift", "Fn"].includes(event.key)) {
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

export const MODIFIER_DOUBLE_TAP_GAP_MS = 500;

const KEY_MODIFIERS = {
  Meta: "Command",
  Alt: "Option",
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

export function modifierNameFromEvent(event) {
  if (!event) {
    return "";
  }
  return KEY_MODIFIERS[event.key] || CODE_MODIFIERS[event.code] || "";
}

export function doubleTapFromModifierEvents(previous, event, now) {
  if (recorderIgnores(event)) {
    return { previous, chord: null };
  }
  const modifier = modifierNameFromEvent(event);
  if (!modifier) {
    return { previous: null, chord: null };
  }
  if (
    previous &&
    previous.modifier === modifier &&
    now - previous.at <= MODIFIER_DOUBLE_TAP_GAP_MS
  ) {
    return {
      previous: null,
      chord: {
        trigger: "modifier_double_tap",
        modifiers: [modifier],
        logicalKey: null,
      },
    };
  }
  return { previous: { modifier, at: now }, chord: null };
}

export function formatModifierDoubleTap(modifier, catalog = {}) {
  const key = DOUBLE_TAP_KEYS[modifier];
  const fallback = DOUBLE_TAP_FALLBACK[modifier] || DOUBLE_TAP_FALLBACK.Shift;
  return (key && catalog[key]) || fallback;
}

export function formatShortcutChord(row, catalog = {}) {
  if (!row?.enabled || row.trigger === "disabled") {
    return catalog["settings.shortcuts.unassigned"] || "Not assigned";
  }
  if (row.trigger === "modifier_double_tap") {
    return formatModifierDoubleTap(row.modifiers?.[0], catalog);
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
  "Type a shortcut or double-tap a modifier (Escape cancels).";
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
  slot.textContent = recordingLabel();
}

function endRecordingChord(item) {
  const slot = item.querySelector("[data-slot='shortcut-chord']");
  if (slot && item.dataset.idleChord != null) {
    slot.textContent = item.dataset.idleChord;
  }
  delete item.dataset.idleChord;
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
        chord.textContent = recordingLabel();
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

export async function bindShortcutRegistry(root, invokeFn) {
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

  const record = async (action, chord, skipTest) => {
    if (isGlobalShortcutAction(action) && chord.modifiers.length === 0) {
      setLive(root, "settings.shortcuts.rejected", REJECTED_FALLBACK);
      await refresh();
      return;
    }
    modifierTap = null;
    delete list.dataset.recordingAction;
    try {
      const rows = await invokeFn("record_shortcut", {
        input: {
          action,
          trigger: chord.trigger,
          modifiers: chord.modifiers,
          logicalKey: chord.logicalKey,
          skipTest,
        },
      });
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
      modifierTap = null;
      list.dataset.recordingAction = action;
      setRecordingRow(root, action);
      recordButton.focus();
      setLive(root, "settings.shortcuts.recording", RECORDING_FALLBACK);
    }
  });

  list.addEventListener("keydown", (event) => {
    const action = list.dataset.recordingAction;
    if (!action) {
      return;
    }
    if (recorderIgnores(event)) {
      return;
    }
    const chord = chordFromKeyboardEvent(event);
    event.preventDefault();
    if (chord?.cancel) {
      modifierTap = null;
      closeRecorder(root);
      setLive(root, "settings.shortcuts.live", LIVE_FALLBACK);
      return;
    }
    if (!chord && modifierNameFromEvent(event)) {
      const now =
        typeof event.timeStamp === "number" && event.timeStamp > 0
          ? event.timeStamp
          : Date.now();
      const tapped = doubleTapFromModifierEvents(modifierTap, event, now);
      modifierTap = tapped.previous;
      if (tapped.chord) {
        record(action, tapped.chord, true);
      }
      return;
    }
    if (!chord) {
      return;
    }
    modifierTap = null;
    record(action, chord, true);
  });

  await refresh();
  return refresh;
}
