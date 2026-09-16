import { useMemo, useState } from "react";
import { Button } from "@/components/button";
import {
  defaultSettingsDraft,
  parseBackupSchedule,
  previewSettingsExport,
  resetSettingsField,
  resetSettingsGroup,
  type SettingsDraft,
  type SettingsFieldId,
  type SettingsGroupId,
  searchSettingsFields,
} from "@/lib/settings-schema";

export const SETTINGS_KEYS = {
  title: "settings.title",
  search: "settings.search.label",
  general: "settings.group.general",
  capture: "settings.group.capture",
  panel: "settings.group.panel",
  copy: "settings.group.copy",
  privacy: "settings.group.privacy",
  data: "settings.group.data",
  accessibility: "settings.group.accessibility",
  launchAtLogin: "settings.field.launchAtLogin",
  backupSchedule: "settings.field.backupSchedule",
  excludedBundleIds: "settings.field.excludedBundleIds",
  daily: "settings.backup.daily",
  weekly: "settings.backup.weekly",
  resetField: "settings.reset.field",
  resetGroup: "settings.reset.group",
  resetAll: "settings.reset.all",
  exportPreview: "settings.export.preview",
  exportSensitive: "settings.export.sensitive",
} as const;

const GROUP_LABEL: Record<SettingsGroupId, keyof typeof SETTINGS_KEYS> = {
  general: "general",
  capture: "capture",
  panel: "panel",
  copy: "copy",
  privacy: "privacy",
  data: "data",
  accessibility: "accessibility",
};

export function SettingsForm({
  labels,
  extraRaw = {},
}: {
  labels: Record<keyof typeof SETTINGS_KEYS, string>;
  extraRaw?: Record<string, string>;
}) {
  const [query, setQuery] = useState("");
  const [draft, setDraft] = useState<SettingsDraft>(defaultSettingsDraft);
  const visible = searchSettingsFields(query);
  const groups = useMemo(() => {
    const seen = new Set<SettingsGroupId>();
    const ordered: SettingsGroupId[] = [];
    for (const field of visible) {
      if (!seen.has(field.group)) {
        seen.add(field.group);
        ordered.push(field.group);
      }
    }
    return ordered;
  }, [visible]);
  const preview = previewSettingsExport(draft, extraRaw);
  const visibleIds = new Set(visible.map((field) => field.id));

  function resetField(id: SettingsFieldId) {
    setDraft((current) => resetSettingsField(current, id));
  }

  return (
    <main data-slot="settings">
      <h1>{labels.title}</h1>
      <search>
        <label htmlFor="settings-search">{labels.search}</label>
        <input
          id="settings-search"
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
      </search>
      <form
        onSubmit={(event) => {
          event.preventDefault();
        }}
      >
        {groups.map((group) => (
          <fieldset key={group} data-settings-group={group}>
            <legend>{labels[GROUP_LABEL[group]]}</legend>
            {visibleIds.has("general.launchAtLogin") && group === "general" ? (
              <p>
                <label htmlFor="launch-at-login">{labels.launchAtLogin}</label>
                <input
                  id="launch-at-login"
                  type="checkbox"
                  checked={draft.launchAtLogin}
                  onChange={(event) =>
                    setDraft((current) => ({
                      ...current,
                      launchAtLogin: event.target.checked,
                    }))
                  }
                />
                <Button
                  type="button"
                  onPress={() => resetField("general.launchAtLogin")}
                >
                  {labels.resetField}
                </Button>
              </p>
            ) : null}
            {visibleIds.has("data.backupSchedule") && group === "data" ? (
              <p>
                <label htmlFor="backup-schedule">{labels.backupSchedule}</label>
                <select
                  id="backup-schedule"
                  value={draft.backupSchedule}
                  onChange={(event) => {
                    const next = parseBackupSchedule(event.target.value);
                    if (next) {
                      setDraft((current) => ({
                        ...current,
                        backupSchedule: next,
                      }));
                    }
                  }}
                >
                  <option value="daily">{labels.daily}</option>
                  <option value="weekly">{labels.weekly}</option>
                </select>
                <Button
                  type="button"
                  onPress={() => resetField("data.backupSchedule")}
                >
                  {labels.resetField}
                </Button>
              </p>
            ) : null}
            {visibleIds.has("privacy.excludedBundleIds") &&
            group === "privacy" ? (
              <p>
                <label htmlFor="excluded-bundle-ids">
                  {labels.excludedBundleIds}
                </label>
                <input
                  id="excluded-bundle-ids"
                  type="text"
                  value={draft.excludedBundleIds}
                  onChange={(event) =>
                    setDraft((current) => ({
                      ...current,
                      excludedBundleIds: event.target.value,
                    }))
                  }
                />
                <Button
                  type="button"
                  onPress={() => resetField("privacy.excludedBundleIds")}
                >
                  {labels.resetField}
                </Button>
              </p>
            ) : null}
            <Button
              type="button"
              onPress={() =>
                setDraft((current) => resetSettingsGroup(current, group))
              }
            >
              {labels.resetGroup}
            </Button>
          </fieldset>
        ))}
        <Button type="button" onPress={() => setDraft(defaultSettingsDraft())}>
          {labels.resetAll}
        </Button>
      </form>
      <section aria-labelledby="settings-export-preview">
        <h2 id="settings-export-preview">{labels.exportPreview}</h2>
        <output data-export-preview>
          <p data-export-schedule>{draft.backupSchedule}</p>
          {preview.sensitiveLiteralKeys.length > 0 ? (
            <p role="status">{labels.exportSensitive}</p>
          ) : null}
          <ul data-export-sensitive-keys>
            {preview.sensitiveLiteralKeys.map((key) => (
              <li key={key}>{key}</li>
            ))}
          </ul>
          <ul data-export-excluded-keys>
            {preview.excludedKeys.map((key) => (
              <li key={key}>{key}</li>
            ))}
          </ul>
          <pre data-export-json>{JSON.stringify(preview.included)}</pre>
        </output>
      </section>
    </main>
  );
}
