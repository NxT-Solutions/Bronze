import { applyHandTestLocale, emitUiLocaleChanged } from "./apply-locale.mjs";
import { runBusy } from "./control.mjs";
import { showChromeWindow, tauriInvoke } from "./tauri-bridge.mjs";

const SWITCHER_LOCALES = ["en", "nl", "fr", "de", "es", "it"];

export function switcherLocale(tag) {
  if (SWITCHER_LOCALES.includes(tag)) {
    return tag;
  }
  return "en";
}

export function settingsSearchNeedle(raw) {
  return String(raw ?? "")
    .trim()
    .toLocaleLowerCase();
}

export function settingsSearchMatches(text, needle) {
  if (!needle) {
    return true;
  }
  return String(text ?? "")
    .toLocaleLowerCase()
    .includes(needle);
}

export function settingsUnitHaystack(unit) {
  const label = unit.querySelector?.("label")?.textContent ?? "";
  const options = [...(unit.querySelectorAll?.("option") ?? [])]
    .map((option) => `${option.textContent ?? ""} ${option.value ?? ""}`)
    .join(" ");
  const control = unit.querySelector?.("input, select, textarea");
  const controlText = control
    ? `${control.value ?? ""} ${control.getAttribute?.("name") ?? ""}`
    : "";
  if (label || options || controlText.trim()) {
    return `${label} ${options} ${controlText}`;
  }
  return unit.textContent ?? "";
}

export function applySettingsSearch(root, rawQuery) {
  const needle = settingsSearchNeedle(rawQuery);
  const searching = needle.length > 0;

  for (const unit of root.querySelectorAll("[data-settings-unit]")) {
    unit.hidden =
      searching && !settingsSearchMatches(settingsUnitHaystack(unit), needle);
  }

  for (const section of root.querySelectorAll("[data-settings-section]")) {
    const title =
      section.querySelector("[data-settings-title]")?.textContent ?? "";
    const titleHit = settingsSearchMatches(title, needle);
    const units = [...section.querySelectorAll("[data-settings-unit]")];
    if (searching && titleHit) {
      for (const unit of units) {
        unit.hidden = false;
      }
      section.hidden = false;
      continue;
    }
    if (units.length > 0) {
      section.hidden = searching && units.every((unit) => unit.hidden);
      continue;
    }
    section.hidden =
      searching && !settingsSearchMatches(section.textContent ?? "", needle);
  }

  const form = root.querySelector("[data-settings-form]");
  if (form) {
    const groups = [...form.querySelectorAll("[data-settings-section]")];
    form.hidden = searching && groups.every((section) => section.hidden);
  }

  const empty = root.querySelector("[data-settings-search-empty]");
  if (empty) {
    const visible = [...root.querySelectorAll("[data-settings-section]")].some(
      (section) => !section.hidden,
    );
    empty.hidden = !searching || visible;
  }
}

export function applySettingsForm(root, settings) {
  const schedule = root.querySelector("#backup-schedule");
  const excluded = root.querySelector("#excluded-bundle-ids");
  const locale = root.querySelector("#ui-locale");
  if (schedule && settings?.data?.backupSchedule) {
    schedule.value = settings.data.backupSchedule;
  }
  if (excluded) {
    excluded.value = (settings?.privacy?.excludedBundleIds ?? []).join(", ");
  }
  if (locale) {
    locale.value = switcherLocale(settings?.general?.locale);
  }
}

export function patchSettingsFromForm(settings, root) {
  const next = structuredClone(settings);
  if (!next.general) {
    next.general = {};
  }
  const schedule = root.querySelector("#backup-schedule")?.value;
  const excluded = root.querySelector("#excluded-bundle-ids")?.value ?? "";
  const locale = root.querySelector("#ui-locale")?.value;
  if (schedule === "daily" || schedule === "weekly") {
    next.data.backupSchedule = schedule;
  }
  next.privacy.excludedBundleIds = excluded
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
  if (SWITCHER_LOCALES.includes(locale)) {
    next.general.locale = locale;
  }
  return next;
}

async function applySavedLocale(root, settings, invokeFn) {
  applySettingsForm(root, settings);
  try {
    const catalog = await invokeFn("ui_catalog");
    applyHandTestLocale(root, settings?.general?.locale, catalog);
  } catch {
    applyHandTestLocale(root, switcherLocale(settings?.general?.locale));
  }
}

export async function bindSettingsLive(
  root = document,
  invokeFn = tauriInvoke,
) {
  const search = root.querySelector("#settings-search");
  const filterSettings = () => {
    const query = search?.value ?? "";
    search
      ?.closest?.(".chrome-search")
      ?.classList.toggle("is-filled", query.trim().length > 0);
    applySettingsSearch(root, query);
  };
  search?.addEventListener("input", filterSettings);
  filterSettings();

  const form = root.querySelector("#settings form");
  if (!form) {
    return;
  }
  let settings;
  try {
    settings = await invokeFn("load_settings_v1");
    await applySavedLocale(root, settings, invokeFn);
  } catch {
    return;
  }

  async function persist() {
    const before = settings?.general?.locale;
    settings = await invokeFn("save_settings_v1", {
      settings: patchSettingsFromForm(settings, root),
    });
    await applySavedLocale(root, settings, invokeFn);
    if (settings?.general?.locale !== before) {
      await emitUiLocaleChanged({ locale: settings.general.locale });
    }
  }

  root.querySelector("#backup-schedule")?.addEventListener("change", persist);
  root
    .querySelector("#excluded-bundle-ids")
    ?.addEventListener("change", persist);
  root.querySelector("#ui-locale")?.addEventListener("change", persist);

  root.querySelectorAll("[data-reset-field]").forEach((button) => {
    button.addEventListener("click", () => {
      runBusy(button, async () => {
        const fieldId = button.getAttribute("data-reset-field");
        const before = settings?.general?.locale;
        settings = await invokeFn("reset_settings_field", { fieldId });
        await applySavedLocale(root, settings, invokeFn);
        if (settings?.general?.locale !== before) {
          await emitUiLocaleChanged({ locale: settings.general.locale });
        }
      });
    });
  });
  const resetGroup = root.querySelector("[data-reset-group]");
  resetGroup?.addEventListener("click", () => {
    runBusy(resetGroup, async () => {
      const group = resetGroup.getAttribute("data-reset-group");
      const before = settings?.general?.locale;
      settings = await invokeFn("reset_settings_group", { group });
      await applySavedLocale(root, settings, invokeFn);
      if (settings?.general?.locale !== before) {
        await emitUiLocaleChanged({ locale: settings.general.locale });
      }
    });
  });
  const resetAll = root.querySelector("[data-reset-all]");
  resetAll?.addEventListener("click", () => {
    runBusy(resetAll, async () => {
      const before = settings?.general?.locale;
      settings = await invokeFn("reset_settings_all");
      await applySavedLocale(root, settings, invokeFn);
      if (settings?.general?.locale !== before) {
        await emitUiLocaleChanged({ locale: settings.general.locale });
      }
    });
  });

  root.querySelectorAll("[data-open-window]").forEach((button) => {
    button.addEventListener("click", () => {
      runBusy(button, async () => {
        const kind = button.getAttribute("data-open-window");
        if (kind) {
          await showChromeWindow(kind, invokeFn);
        }
      });
    });
  });
}

if (globalThis.document?.readyState) {
  bindSettingsLive();
}
