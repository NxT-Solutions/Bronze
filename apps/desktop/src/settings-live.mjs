import {
  applyHandTestLocale,
  catalogMessage,
  emitUiLocaleChanged,
} from "./apply-locale.mjs";
import { runBusy } from "./control.mjs";
import { sourceIconSrc } from "./item-view.mjs";
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
  const help = unit.querySelector?.("[data-excluded-help]")?.textContent ?? "";
  const howto =
    unit.querySelector?.("[data-excluded-howto]")?.textContent ?? "";
  const chips = [...(unit.querySelectorAll?.("[data-app-name]") ?? [])]
    .map((node) => node.getAttribute?.("data-app-name") ?? "")
    .join(" ");
  const options = [...(unit.querySelectorAll?.("option") ?? [])]
    .map((option) => `${option.textContent ?? ""} ${option.value ?? ""}`)
    .join(" ");
  const control = unit.querySelector?.("input, select, textarea");
  const controlText = control
    ? `${control.value ?? ""} ${control.getAttribute?.("name") ?? ""}`
    : "";
  if (label || help || howto || chips || options || controlText.trim()) {
    return `${label} ${help} ${howto} ${chips} ${options} ${controlText}`;
  }
  return unit.textContent ?? "";
}

export function isSafeDisplayName(name) {
  const value = String(name ?? "").trim();
  return (
    value.length > 0 &&
    !value.includes("/") &&
    !value.includes("\\") &&
    !value.includes("\0") &&
    !value.includes("..")
  );
}

export function pickerFailureCode(error) {
  const text =
    typeof error === "string" ? error : String(error?.message ?? error ?? "");
  if (text.includes("picker_cancelled")) {
    return "picker_cancelled";
  }
  if (text.includes("picker_unavailable")) {
    return "picker_unavailable";
  }
  return "invalid_app";
}

export function addExcludedApp(host, app) {
  if (!isSafeBundleId(app?.bundleId)) {
    return false;
  }
  const bundleId = app.bundleId.trim();
  const rawName = String(app.name ?? "").trim();
  const picked = {
    bundleId,
    name: isSafeDisplayName(rawName) ? rawName : bundleId,
  };
  const catalog = [...(host?._installedApps ?? []), picked];
  if (host) {
    host._installedApps = catalog;
  }
  writeExcludedApps(host, [
    ...resolveExcludedApps(readExcludedBundleIds(host), catalog),
    picked,
  ]);
  return true;
}

export async function pickExcludedApp(host, invokeFn) {
  try {
    const app = await invokeFn("pick_installed_app");
    if (!addExcludedApp(host, app)) {
      return "invalid_app";
    }
    return "picked";
  } catch (error) {
    return pickerFailureCode(error);
  }
}

export function isSafeBundleId(id) {
  const value = String(id ?? "").trim();
  return (
    value.length > 0 &&
    value.length <= 256 &&
    !value.includes("/") &&
    !value.includes("\\") &&
    !value.includes("\0") &&
    !value.includes("..")
  );
}

export function filterInstalledApps(apps, query, selectedIds) {
  const needle = settingsSearchNeedle(query);
  const selected = new Set(
    (selectedIds ?? [])
      .filter(isSafeBundleId)
      .map((id) => id.trim().toLocaleLowerCase()),
  );
  return (Array.isArray(apps) ? apps : []).filter((app) => {
    if (!isSafeBundleId(app?.bundleId)) {
      return false;
    }
    if (selected.has(app.bundleId.trim().toLocaleLowerCase())) {
      return false;
    }
    return settingsSearchMatches(`${app.name ?? ""} ${app.bundleId}`, needle);
  });
}

export function resolveExcludedApps(ids, catalog) {
  const byId = new Map();
  for (const app of Array.isArray(catalog) ? catalog : []) {
    if (!isSafeBundleId(app?.bundleId)) {
      continue;
    }
    byId.set(app.bundleId.trim().toLocaleLowerCase(), {
      bundleId: app.bundleId.trim(),
      name: String(app.name ?? "").trim() || app.bundleId.trim(),
    });
  }
  const out = [];
  const seen = new Set();
  for (const raw of ids ?? []) {
    if (!isSafeBundleId(raw)) {
      continue;
    }
    const key = raw.trim().toLocaleLowerCase();
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    out.push(
      byId.get(key) ?? {
        bundleId: raw.trim(),
        name: raw.trim(),
      },
    );
  }
  return out;
}

export function readExcludedBundleIds(host) {
  const raw = host?.dataset?.excludedIds;
  if (typeof raw === "string") {
    return raw
      .split("\n")
      .map((value) => value.trim())
      .filter(isSafeBundleId);
  }
  return [...(host?.querySelectorAll?.("[data-excluded-app]") ?? [])]
    .map((node) => node.getAttribute?.("data-bundle-id"))
    .filter(isSafeBundleId);
}

export function writeExcludedApps(host, apps) {
  const selected = resolveExcludedApps(
    (apps ?? []).map((app) => app.bundleId),
    apps,
  );
  if (host?.dataset) {
    host.dataset.excludedIds = selected.map((app) => app.bundleId).join("\n");
  }
  renderExcludedChips(host, selected);
  return selected;
}

export function renderExcludedChips(host, apps) {
  const list = host?.querySelector?.("#excluded-apps-selected");
  if (!list?.replaceChildren) {
    return;
  }
  list.replaceChildren();
  const removeLabel =
    catalogMessage("settings.field.excludedBundleIds.remove") || "Remove";
  for (const app of apps) {
    const item = globalThis.document?.createElement?.("li");
    if (!item) {
      continue;
    }
    item.className = "app-chip";
    item.dataset.excludedApp = "";
    item.dataset.bundleId = app.bundleId;
    item.dataset.appName = app.name;
    const icon = globalThis.document.createElement("img");
    icon.alt = "";
    icon.className = "app-picker-option-icon";
    icon.width = 16;
    icon.height = 16;
    const name = globalThis.document.createElement("span");
    name.className = "app-chip-name";
    name.textContent = app.name;
    const remove = globalThis.document.createElement("button");
    remove.type = "button";
    remove.className = "btn-icon";
    remove.dataset.removeExcluded = app.bundleId;
    remove.setAttribute(
      "data-i18n-aria-label",
      "settings.field.excludedBundleIds.remove",
    );
    remove.setAttribute("aria-label", removeLabel);
    const mark = globalThis.document.createElementNS(
      "http://www.w3.org/2000/svg",
      "svg",
    );
    mark.setAttribute("viewBox", "0 0 12 12");
    mark.setAttribute("aria-hidden", "true");
    mark.setAttribute("focusable", "false");
    const path = globalThis.document.createElementNS(
      "http://www.w3.org/2000/svg",
      "path",
    );
    path.setAttribute(
      "d",
      "M2.1 1.4 1.4 2.1 5.3 6 1.4 9.9l.7.7L6 6.7l3.9 3.9.7-.7L6.7 6l3.9-3.9-.7-.7L6 5.3z",
    );
    path.setAttribute("fill", "currentColor");
    mark.append(path);
    remove.append(mark);
    item.append(icon, name, remove);
    list.append(item);
  }
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
  const locale = root.querySelector("#ui-locale");
  const host = root.querySelector("#excluded-apps");
  if (schedule && settings?.data?.backupSchedule) {
    schedule.value = settings.data.backupSchedule;
  }
  if (host) {
    writeExcludedApps(
      host,
      resolveExcludedApps(
        settings?.privacy?.excludedBundleIds ?? [],
        host._installedApps ?? [],
      ),
    );
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
  const locale = root.querySelector("#ui-locale")?.value;
  if (schedule === "daily" || schedule === "weekly") {
    next.data.backupSchedule = schedule;
  }
  next.privacy.excludedBundleIds = readExcludedBundleIds(
    root.querySelector("#excluded-apps"),
  );
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

export function bindSearchClear(root = document) {
  for (const wrap of root.querySelectorAll(".chrome-search")) {
    const input = wrap.querySelector("input[type='search']");
    const clear = wrap.querySelector("[data-search-clear]");
    if (!input || !clear) {
      continue;
    }
    const sync = () => {
      clear.hidden = String(input.value ?? "").length === 0;
    };
    input.addEventListener("input", sync);
    clear.addEventListener("click", () => {
      input.value = "";
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.focus();
    });
    sync();
  }
}

export async function bindSettingsLive(
  root = document,
  invokeFn = tauriInvoke,
) {
  bindSearchClear(root);
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
  } catch {
    return;
  }

  const host = root.querySelector("#excluded-apps");
  const unavailable = root.querySelector("[data-excluded-apps-unavailable]");
  const appSearch = root.querySelector("#excluded-apps-search");
  let installedApps = [];
  let appsUnavailable = false;
  try {
    const listed = await invokeFn("list_installed_apps");
    if (!Array.isArray(listed)) {
      appsUnavailable = true;
    } else {
      installedApps = listed.filter((app) => isSafeBundleId(app?.bundleId));
    }
  } catch {
    appsUnavailable = true;
  }
  if (host) {
    host._installedApps = installedApps;
    host._appsUnavailable = appsUnavailable;
  }
  if (unavailable) {
    unavailable.hidden = !appsUnavailable;
  }
  if (appSearch) {
    appSearch.disabled = appsUnavailable;
  }
  await applySavedLocale(root, settings, invokeFn);
  await refreshExcludedIcons(root, invokeFn);
  bindExcludedPicker(root, invokeFn, () => persist());

  async function persist() {
    const before = settings?.general?.locale;
    settings = await invokeFn("save_settings_v1", {
      settings: patchSettingsFromForm(settings, root),
    });
    await applySavedLocale(root, settings, invokeFn);
    await refreshExcludedIcons(root, invokeFn);
    if (settings?.general?.locale !== before) {
      await emitUiLocaleChanged({ locale: settings.general.locale });
    }
  }

  root.querySelector("#backup-schedule")?.addEventListener("change", persist);
  root.querySelector("#ui-locale")?.addEventListener("change", persist);

  root.querySelectorAll("[data-reset-field]").forEach((button) => {
    button.addEventListener("click", () => {
      runBusy(button, async () => {
        const fieldId = button.getAttribute("data-reset-field");
        const before = settings?.general?.locale;
        settings = await invokeFn("reset_settings_field", { fieldId });
        await applySavedLocale(root, settings, invokeFn);
        await refreshExcludedIcons(root, invokeFn);
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
      await refreshExcludedIcons(root, invokeFn);
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
      await refreshExcludedIcons(root, invokeFn);
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

function showPickerStatus(root, code) {
  const status = root.querySelector("[data-excluded-apps-picker]");
  if (!status) {
    return;
  }
  if (code === "picked" || code === "picker_cancelled") {
    status.textContent = "";
    status.hidden = true;
    return;
  }
  const key =
    code === "picker_unavailable"
      ? "settings.field.excludedBundleIds.pickerUnavailable"
      : "settings.field.excludedBundleIds.invalidApp";
  status.textContent =
    catalogMessage(key) ||
    (code === "picker_unavailable"
      ? "The app picker is unavailable."
      : "That item is not a valid app.");
  status.hidden = false;
}

function bindExcludedPicker(root, invokeFn, persist) {
  const host = root.querySelector("#excluded-apps");
  const search = root.querySelector("#excluded-apps-search");
  const list = root.querySelector("#excluded-apps-list");
  const empty = root.querySelector("[data-excluded-apps-empty]");
  const choose = root.querySelector("[data-pick-installed-app]");
  const searchWrap = search?.closest?.(".app-picker-search");
  if (!host || !search || !list) {
    return;
  }

  const syncSearchChrome = () => {
    searchWrap?.classList.toggle("is-filled", search.value.trim().length > 0);
  };

  const closeList = () => {
    list.hidden = true;
    search.setAttribute("aria-expanded", "false");
    if (empty) {
      empty.hidden = true;
    }
  };

  const paintOptions = () => {
    if (host._appsUnavailable) {
      closeList();
      return;
    }
    const matches = filterInstalledApps(
      host._installedApps ?? [],
      search.value,
      readExcludedBundleIds(host),
    );
    list.replaceChildren();
    for (const app of matches.slice(0, 50)) {
      const option = globalThis.document.createElement("button");
      option.type = "button";
      option.setAttribute("role", "option");
      option.className = "app-picker-option";
      option.dataset.bundleId = app.bundleId;
      option.dataset.appName = app.name;
      const icon = globalThis.document.createElement("img");
      icon.alt = "";
      icon.className = "app-picker-option-icon";
      icon.width = 16;
      icon.height = 16;
      const label = globalThis.document.createElement("span");
      label.textContent = app.name;
      option.append(icon, label);
      option.addEventListener("click", () => {
        writeExcludedApps(host, [
          ...resolveExcludedApps(
            readExcludedBundleIds(host),
            host._installedApps,
          ),
          app,
        ]);
        search.value = "";
        syncSearchChrome();
        closeList();
        persist();
      });
      list.append(option);
      fillAppIcon(icon, app.bundleId, invokeFn);
    }
    const searching = search.value.trim().length > 0;
    list.hidden = !searching || matches.length === 0;
    search.setAttribute(
      "aria-expanded",
      searching && matches.length > 0 ? "true" : "false",
    );
    if (empty) {
      empty.hidden = !searching || matches.length > 0;
    }
    for (const chip of host.querySelectorAll("[data-excluded-app] img")) {
      fillAppIcon(chip, chip.parentElement?.dataset?.bundleId, invokeFn);
    }
  };

  search.addEventListener("input", () => {
    syncSearchChrome();
    paintOptions();
  });
  search.addEventListener("focus", paintOptions);
  choose?.addEventListener("click", () => {
    runBusy(choose, async () => {
      const result = await pickExcludedApp(host, invokeFn);
      showPickerStatus(root, result);
      if (result === "picked") {
        search.value = "";
        syncSearchChrome();
        closeList();
        persist();
      }
    });
  });
  search.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      closeList();
    }
    if (event.key === "Enter") {
      event.preventDefault();
      list.querySelector("[data-bundle-id]")?.click();
    }
  });
  host.addEventListener("click", (event) => {
    const button = event.target?.closest?.("[data-remove-excluded]");
    if (!button || !host.contains(button)) {
      return;
    }
    const removeId = button.getAttribute("data-remove-excluded");
    writeExcludedApps(
      host,
      resolveExcludedApps(
        readExcludedBundleIds(host),
        host._installedApps,
      ).filter(
        (app) =>
          app.bundleId.toLocaleLowerCase() !== removeId?.toLocaleLowerCase(),
      ),
    );
    persist();
  });
  root.addEventListener?.("pointerdown", (event) => {
    if (!host.contains(event.target)) {
      closeList();
    }
  });
  syncSearchChrome();
  paintOptions();
}

async function refreshExcludedIcons(root, invokeFn) {
  const host = root.querySelector("#excluded-apps");
  if (!host) {
    return;
  }
  for (const chip of host.querySelectorAll("[data-excluded-app] img")) {
    await fillAppIcon(chip, chip.parentElement?.dataset?.bundleId, invokeFn);
  }
}

async function fillAppIcon(img, bundleId, invokeFn) {
  if (!img || !isSafeBundleId(bundleId)) {
    return;
  }
  try {
    const src = sourceIconSrc(
      await invokeFn("app_icon_data_url", { bundleId }),
    );
    if (src) {
      img.src = src;
      img.classList.add("is-ready");
    } else {
      img.classList.remove("is-ready");
    }
  } catch {
    img.classList.remove("is-ready");
  }
}

if (globalThis.document?.readyState) {
  bindSettingsLive();
}
