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

  const search = root.querySelector("#settings-search");
  search?.addEventListener("input", async () => {
    const ids = await invokeFn("search_settings_fields", {
      query: search.value,
    });
    for (const fieldset of root.querySelectorAll("[data-settings-group]")) {
      const group = fieldset.getAttribute("data-settings-group");
      fieldset.hidden =
        search.value.length > 0 &&
        !ids.some((id) => id.startsWith(`${group}.`));
    }
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
