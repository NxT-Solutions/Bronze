export const SHORTCUT_ACTION_IDS = [
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
] as const;

export type ShortcutActionId = (typeof SHORTCUT_ACTION_IDS)[number];

export type ShortcutBinding = {
  action: ShortcutActionId;
  chord: string;
  enabled: boolean;
};

export type CaptureAlternatives = {
  chord: boolean;
  menu: boolean;
  manual: boolean;
};

export function captureAlternativesOk(alt: CaptureAlternatives): boolean {
  return alt.chord || alt.menu || alt.manual;
}

export function standardChordAction(): ShortcutActionId {
  return "capture.selection";
}

export function recorderSwallows(event: {
  isComposing: boolean;
  isVoiceOverReserved: boolean;
  isRepeat: boolean;
}): boolean {
  return !event.isComposing && !event.isVoiceOverReserved && !event.isRepeat;
}

export const DEFAULT_SHORTCUT_CHORDS: Record<ShortcutActionId, string> = {
  "app.togglePanel": "Option+Space",
  "capture.selection": "Shift-double-tap",
  "capture.newNote": "Command+N",
  "queue.copy": "Command+C",
  "queue.copyWithProfile": "Command+Shift+C",
  "queue.copyAndAdvance": "Command+Shift+Enter",
  "queue.complete": "Space",
  "queue.edit": "Enter",
  "queue.moveUp": "Option+Command+ArrowUp",
  "queue.moveDown": "Option+Command+ArrowDown",
  "queue.search": "Command+F",
  "queue.undo": "Command+Z",
  "window.settings": "Command+,",
};

export function seedShortcutRegistry(): ShortcutBinding[] {
  return SHORTCUT_ACTION_IDS.map((action) => ({
    action,
    chord: DEFAULT_SHORTCUT_CHORDS[action],
    enabled: true,
  }));
}

export function registerShortcut(
  current: ShortcutBinding[],
  candidate: ShortcutBinding,
  nativeOk: boolean,
  alternatives: CaptureAlternatives,
): ShortcutBinding[] {
  if (
    candidate.action === "capture.selection" &&
    !candidate.enabled &&
    !captureAlternativesOk({ ...alternatives, chord: false })
  ) {
    return current;
  }
  if (!nativeOk) {
    return current;
  }
  return current.map((binding) =>
    binding.action === candidate.action ? candidate : binding,
  );
}
