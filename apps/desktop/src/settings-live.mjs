import { showChromeWindow, tauriInvoke } from "./tauri-bridge.mjs";

export function applySettingsForm(root, settings) {
  const schedule = root.querySelector("#backup-schedule");
  const excluded = root.querySelector("#excluded-bundle-ids");
  if (schedule && settings?.data?.backupSchedule) {
    schedule.value = settings.data.backupSchedule;
  }
  if (excluded) {
    excluded.value = (settings?.privacy?.excludedBundleIds ?? []).join(", ");
  }
}

export function patchSettingsFromForm(settings, root) {
  const next = structuredClone(settings);
  const schedule = root.querySelector("#backup-schedule")?.value;
  const excluded = root.querySelector("#excluded-bundle-ids")?.value ?? "";
  if (schedule === "daily" || schedule === "weekly") {
    next.data.backupSchedule = schedule;
  }
  next.privacy.excludedBundleIds = excluded
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
  return next;
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
    applySettingsForm(root, settings);
  } catch {
    return;
  }

  async function persist() {
    settings = await invokeFn("save_settings_v1", {
      settings: patchSettingsFromForm(settings, root),
    });
    applySettingsForm(root, settings);
  }

  root.querySelector("#backup-schedule")?.addEventListener("change", persist);
  root
    .querySelector("#excluded-bundle-ids")
    ?.addEventListener("change", persist);

  root.querySelectorAll("[data-reset-field]").forEach((button) => {
    button.addEventListener("click", async () => {
      const fieldId = button.getAttribute("data-reset-field");
      settings = await invokeFn("reset_settings_field", { fieldId });
      applySettingsForm(root, settings);
    });
  });
  root
    .querySelector("[data-reset-group]")
    ?.addEventListener("click", async () => {
      const group = root
        .querySelector("[data-reset-group]")
        .getAttribute("data-reset-group");
      settings = await invokeFn("reset_settings_group", { group });
      applySettingsForm(root, settings);
    });
  root
    .querySelector("[data-reset-all]")
    ?.addEventListener("click", async () => {
      settings = await invokeFn("reset_settings_all");
      applySettingsForm(root, settings);
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
      const kind = button.getAttribute("data-open-window");
      if (kind) {
        showChromeWindow(kind, invokeFn);
      }
    });
  });
}

if (globalThis.document?.readyState) {
  bindSettingsLive();
}
