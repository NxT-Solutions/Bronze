import { tauriInvoke } from "./tauri-bridge.mjs";

export const HAND_TEST_DEFAULT_LOCALE = "en";

export const EN_HAND_TEST_CATALOG = {
  "settings.title": "Settings",
  "settings.search.label": "Search settings",
  "settings.group.data": "Data",
  "settings.group.privacy": "Privacy",
  "settings.field.backupSchedule": "Backup schedule",
  "settings.field.excludedBundleIds": "Excluded apps",
  "settings.backup.daily": "Daily",
  "settings.backup.weekly": "Weekly",
  "settings.reset.field": "Reset field",
  "settings.reset.group": "Reset group",
  "settings.reset.all": "Reset all",
  "settings.export.preview": "Export preview",
  "settings.export.sensitive": "User-entered literals may be sensitive",
  "settings.shortcuts.title": "Shortcuts",
  "settings.shortcuts.record": "Record shortcut",
  "settings.shortcuts.skipTest": "Skip test",
  "settings.shortcuts.live": "Listening",
  "settings.shortcuts.unassigned": "Not assigned",
  "settings.shortcuts.action.app.togglePanel": "Toggle panel",
  "settings.shortcuts.action.capture.selection": "Capture selection",
  "settings.shortcuts.action.capture.newNote": "New note",
  "settings.shortcuts.action.queue.copy": "Copy",
  "settings.shortcuts.action.queue.copyWithProfile": "Copy with profile",
  "settings.shortcuts.action.queue.copyAndAdvance": "Copy and advance",
  "settings.shortcuts.action.queue.complete": "Complete",
  "settings.shortcuts.action.queue.edit": "Edit",
  "settings.shortcuts.action.queue.moveUp": "Move up",
  "settings.shortcuts.action.queue.moveDown": "Move down",
  "settings.shortcuts.action.queue.search": "Search",
  "settings.shortcuts.action.queue.undo": "Undo",
  "settings.shortcuts.action.window.settings": "Open settings",
  "settings.permission.title": "Permission health",
  "settings.permission.retest": "Retest",
  "settings.permission.status.denied": "Denied",
  "settings.permission.inputMonitoring.why":
    "Needed for the global capture chord",
  "settings.permission.inputMonitoring.alternative": "Use the menu or composer",
  "settings.permission.accessibility.why":
    "Needed to read the current selection",
  "settings.permission.accessibility.alternative":
    "Paste or type into the composer",
  "settings.permission.screenRecording.why":
    "Bronze does not record the screen",
  "settings.permission.screenRecording.notUsed": "Not used",
  "settings.permission.composer.available": "Manual composer remains available",
  "settings.permission.openSystemSettings": "Open System Settings",
  "help.title": "Help",
  "library.title": "Library",
  "panel.quick.title": "Bronze",
  "panel.section.active": "Inbox",
  "panel.toolbar.overflow": "More actions",
  "composer.add.label": "Add item",
  "composer.add.submit": "Add",
  "composer.add.error": "Could not add item",
  "queue.item.moveUp": "Move up",
  "queue.item.moveDown": "Move down",
  "queue.item.complete": "Complete",
  "queue.item.skip": "Skip",
  "queue.item.trash": "Trash",
  "queue.item.edit": "Edit",
  "copy.profile.label": "Output profile",
  "copy.action.copy": "Copy",
  "copy.preview.label": "Preview",
};

export function resolveUiLocale(requested) {
  if (requested === "en-XA" || requested === "ar-XB") {
    return requested;
  }
  return HAND_TEST_DEFAULT_LOCALE;
}

export function catalogHasRealSpaces(value) {
  return typeof value === "string" && /\s/.test(value);
}

export function expandHandTestString(locale, value) {
  const resolved = resolveUiLocale(locale);
  if (resolved === HAND_TEST_DEFAULT_LOCALE) {
    return value;
  }
  return value;
}

export function looksSmashedLocale(value, current = "") {
  if (typeof value !== "string" || value.length === 0) {
    return true;
  }
  if (value.startsWith("【") && value.endsWith("】")) {
    return false;
  }
  if (catalogHasRealSpaces(current.trim()) && !catalogHasRealSpaces(value)) {
    return true;
  }
  return !catalogHasRealSpaces(value) && /(\p{L})\1/u.test(value);
}

export function applyCatalogStrings(root, catalog) {
  if (!catalog || typeof root?.querySelectorAll !== "function") {
    return;
  }
  for (const node of root.querySelectorAll("[data-i18n]")) {
    const key = node.getAttribute("data-i18n");
    const value = catalog[key];
    if (typeof value !== "string" || value.length === 0) {
      continue;
    }
    const current = node.textContent ?? "";
    if (looksSmashedLocale(value, current)) {
      continue;
    }
    if (current.trim() === value) {
      continue;
    }
    node.textContent = value;
  }
}

export function watchEnglishCatalog(root, catalog) {
  if (typeof MutationObserver !== "function") {
    return;
  }
  const target = root.body ?? root;
  if (!target || typeof target.addEventListener !== "function") {
    return;
  }
  const observer = new MutationObserver(() => {
    applyCatalogStrings(root, catalog);
  });
  try {
    observer.observe(target, {
      subtree: true,
      characterData: true,
      childList: true,
    });
  } catch {
    observer.disconnect();
  }
}

export function applyHandTestLocale(
  root = document,
  locale = HAND_TEST_DEFAULT_LOCALE,
  catalog,
) {
  const resolved = resolveUiLocale(locale);
  const html = root.documentElement ?? root;
  html.lang = resolved === "ar-XB" ? "ar" : "en";
  html.dir = resolved === "ar-XB" ? "rtl" : "ltr";
  if (resolved === HAND_TEST_DEFAULT_LOCALE) {
    applyCatalogStrings(root, catalog ?? EN_HAND_TEST_CATALOG);
  }
  return resolved;
}

export async function bindHandTestLocale(
  root = document,
  invokeFn = tauriInvoke,
) {
  let requested = HAND_TEST_DEFAULT_LOCALE;
  try {
    requested = await invokeFn("ui_locale");
  } catch {
    requested = HAND_TEST_DEFAULT_LOCALE;
  }
  const resolved = applyHandTestLocale(root, requested);
  if (resolved === HAND_TEST_DEFAULT_LOCALE) {
    watchEnglishCatalog(root, EN_HAND_TEST_CATALOG);
  }
  return resolved;
}

if (globalThis.document?.readyState) {
  bindHandTestLocale();
}
