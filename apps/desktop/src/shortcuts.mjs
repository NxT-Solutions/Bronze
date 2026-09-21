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
  if (event.key === "Escape") {
    return { cancel: true };
  }
  if (["Meta", "Alt", "Control", "Shift", "Fn"].includes(event.key)) {
    return null;
  }
  const logicalKey = normalizeLogicalKey(event.key);
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

export function formatShortcutChord(row, catalog = {}) {
  if (!row?.enabled || row.trigger === "disabled") {
    return catalog["settings.shortcuts.unassigned"] || "Not assigned";
  }
  if (row.trigger === "modifier_double_tap") {
    return (
      catalog["settings.shortcuts.chord.shiftDoubleTap"] || "Shift double-tap"
    );
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
const RECORDING_FALLBACK = "Type the new shortcut (Escape cancels).";
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
      chord.textContent = formatShortcutChord(row, labels);
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
    } else {
      delete item.dataset.recording;
    }
    item
      .querySelector("[data-shortcut-record]")
      ?.setAttribute("aria-pressed", match ? "true" : "false");
  }
}

function closeRecorder(root) {
  const bar = root.querySelector("[data-shortcut-record-bar]");
  const input = root.querySelector("#shortcut-record");
  if (bar) {
    bar.hidden = true;
    delete bar.dataset.recordingAction;
    delete bar.dataset.pendingChord;
  }
  if (input) {
    input.value = "";
  }
  setRecordingRow(root, "");
}

function pendingChord(bar) {
  try {
    const chord = JSON.parse(bar?.dataset.pendingChord || "");
    if (chord?.trigger && typeof chord.logicalKey === "string") {
      return chord;
    }
  } catch {
    return null;
  }
  return null;
}

function stageChord(bar, input, chord) {
  if (!bar) {
    return;
  }
  bar.dataset.pendingChord = JSON.stringify(chord);
  if (input) {
    input.value = formatShortcutChord(
      {
        enabled: true,
        trigger: chord.trigger,
        modifiers: chord.modifiers,
        logicalKey: chord.logicalKey,
      },
      catalog(),
    );
  }
}

export async function bindShortcutRegistry(root, invokeFn) {
  const list = root.querySelector("[data-shortcut-registry]");
  const bar = root.querySelector("[data-shortcut-record-bar]");
  const input = root.querySelector("#shortcut-record");
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

  const record = async (action, chord, skipTest) => {
    if (isGlobalShortcutAction(action) && chord.modifiers.length === 0) {
      setLive(root, "settings.shortcuts.rejected", REJECTED_FALLBACK);
      return;
    }
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
      setLive(root, shortcutFailureKey(error), REJECTED_FALLBACK);
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
    if (recordButton && bar && input) {
      bar.hidden = false;
      bar.dataset.recordingAction = action;
      delete bar.dataset.pendingChord;
      input.value = "";
      setRecordingRow(root, action);
      input.focus();
      setLive(root, "settings.shortcuts.recording", RECORDING_FALLBACK);
    }
  });

  input?.addEventListener("keydown", (event) => {
    const action = bar?.dataset.recordingAction;
    if (!action) {
      return;
    }
    const chord = chordFromKeyboardEvent(event);
    if (!chord) {
      return;
    }
    event.preventDefault();
    if (chord.cancel) {
      closeRecorder(root);
      setLive(root, "settings.shortcuts.live", LIVE_FALLBACK);
      return;
    }
    stageChord(bar, input, chord);
    record(action, chord, true);
  });

  root.querySelector("[data-shortcut-skip]")?.addEventListener("click", () => {
    const action = bar?.dataset.recordingAction;
    const chord = pendingChord(bar);
    if (action && chord) {
      record(action, chord, true);
      return;
    }
    closeRecorder(root);
    setLive(root, "settings.shortcuts.live", LIVE_FALLBACK);
  });
  root
    .querySelector("[data-shortcut-cancel]")
    ?.addEventListener("click", () => {
      closeRecorder(root);
      setLive(root, "settings.shortcuts.live", LIVE_FALLBACK);
    });

  await refresh();
  return refresh;
}
