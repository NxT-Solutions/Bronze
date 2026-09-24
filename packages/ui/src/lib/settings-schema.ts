export type BackupSchedule = "daily" | "weekly";
export type TitleModelId =
  | ""
  | "extractive"
  | "smol-135"
  | "smol-360"
  | "qwen-05"
  | "custom"
  | "ollama"
  | "hosted-openai"
  | "hosted-anthropic"
  | "hosted-openrouter";
export type SettingsGroupId =
  | "general"
  | "capture"
  | "panel"
  | "copy"
  | "privacy"
  | "data"
  | "accessibility";

export type SettingsFieldId =
  | "general.launchAtLogin"
  | "general.titleModel"
  | "capture.standardChord"
  | "privacy.excludedBundleIds"
  | "privacy.appPolicies"
  | "data.backupSchedule";

export type SettingsFieldDef = {
  id: SettingsFieldId;
  group: SettingsGroupId;
  tokens: string[];
};

export const SETTINGS_FIELDS: readonly SettingsFieldDef[] = [
  {
    id: "general.launchAtLogin",
    group: "general",
    tokens: ["launch", "login"],
  },
  {
    id: "general.titleModel",
    group: "general",
    tokens: [
      "title",
      "model",
      "smol",
      "qwen",
      "extractive",
      "engine",
      "ollama",
      "hosted",
      "custom",
      "gguf",
    ],
  },
  {
    id: "capture.standardChord",
    group: "capture",
    tokens: ["shortcut", "chord", "capture"],
  },
  {
    id: "privacy.excludedBundleIds",
    group: "privacy",
    tokens: ["exclude", "bundle"],
  },
  {
    id: "privacy.appPolicies",
    group: "privacy",
    tokens: ["policy", "policies"],
  },
  {
    id: "data.backupSchedule",
    group: "data",
    tokens: ["backup", "daily", "weekly"],
  },
];

export type SettingsDraft = {
  launchAtLogin: boolean;
  titleModel: TitleModelId;
  titleCustomId: string;
  titleCustomName: string;
  titleCustomBytes: number;
  titleOllamaModel: string;
  titleHostedBase: string;
  titleHostedConfirmed: boolean;
  backupSchedule: BackupSchedule;
  excludedBundleIds: string;
  appPolicies: string;
  standardChordEnabled: boolean;
};

export function defaultSettingsDraft(): SettingsDraft {
  return {
    launchAtLogin: false,
    titleModel: "",
    titleCustomId: "",
    titleCustomName: "",
    titleCustomBytes: 0,
    titleOllamaModel: "",
    titleHostedBase: "",
    titleHostedConfirmed: false,
    backupSchedule: "daily",
    excludedBundleIds: "",
    appPolicies: "",
    standardChordEnabled: true,
  };
}

export function parseTitleModelId(raw: string): TitleModelId | null {
  if (
    raw === "" ||
    raw === "extractive" ||
    raw === "smol-135" ||
    raw === "smol-360" ||
    raw === "qwen-05" ||
    raw === "custom" ||
    raw === "ollama" ||
    raw === "hosted-openai" ||
    raw === "hosted-anthropic" ||
    raw === "hosted-openrouter"
  ) {
    return raw;
  }
  return null;
}

export function parseBackupSchedule(raw: string): BackupSchedule | null {
  if (raw === "daily" || raw === "weekly") {
    return raw;
  }
  return null;
}

export function searchSettingsFields(query: string): SettingsFieldDef[] {
  const q = query.trim().toLowerCase();
  return SETTINGS_FIELDS.filter((field) => {
    if (q.length === 0) {
      return true;
    }
    return (
      field.id.toLowerCase().includes(q) ||
      field.group.includes(q) ||
      field.tokens.some((token) => token.includes(q) || q.includes(token))
    );
  });
}

export function resetSettingsField(
  draft: SettingsDraft,
  fieldId: SettingsFieldId,
): SettingsDraft {
  const defaults = defaultSettingsDraft();
  switch (fieldId) {
    case "general.launchAtLogin":
      return { ...draft, launchAtLogin: defaults.launchAtLogin };
    case "general.titleModel":
      return {
        ...draft,
        titleModel: defaults.titleModel,
        titleCustomId: defaults.titleCustomId,
        titleCustomName: defaults.titleCustomName,
        titleCustomBytes: defaults.titleCustomBytes,
        titleOllamaModel: defaults.titleOllamaModel,
        titleHostedBase: defaults.titleHostedBase,
        titleHostedConfirmed: defaults.titleHostedConfirmed,
      };
    case "capture.standardChord":
      return { ...draft, standardChordEnabled: defaults.standardChordEnabled };
    case "privacy.excludedBundleIds":
      return { ...draft, excludedBundleIds: defaults.excludedBundleIds };
    case "privacy.appPolicies":
      return { ...draft, appPolicies: defaults.appPolicies };
    case "data.backupSchedule":
      return { ...draft, backupSchedule: defaults.backupSchedule };
    default:
      return draft;
  }
}

export function resetSettingsGroup(
  draft: SettingsDraft,
  group: SettingsGroupId,
): SettingsDraft {
  let next = draft;
  for (const field of SETTINGS_FIELDS) {
    if (field.group === group) {
      next = resetSettingsField(next, field.id);
    }
  }
  return next;
}

const FORBIDDEN = [
  "credential",
  "token",
  "path",
  "secret",
  "installidentity",
  "diagnosticevent",
  "apikey",
  "hostedkey",
];

export function settingsKeyExportable(key: string): boolean {
  const lower = key.toLowerCase().replace(/[._-]/g, "");
  return !FORBIDDEN.some((frag) => lower.includes(frag));
}

export function valueLooksLikeMachinePath(value: string): boolean {
  const trimmed = value.trim().replace(/^"|"$/g, "");
  return trimmed.startsWith("/") || trimmed.includes(":\\");
}

export type SettingsExportPreview = {
  included: Record<string, string>;
  sensitiveLiteralKeys: string[];
  excludedKeys: string[];
};

export function previewSettingsExport(
  draft: SettingsDraft,
  extraRaw: Record<string, string> = {},
): SettingsExportPreview {
  const included: Record<string, string> = {
    "data.backupSchedule": draft.backupSchedule,
    "general.launchAtLogin": String(draft.launchAtLogin),
  };
  if (draft.titleModel) {
    included["general.titleModel"] = draft.titleModel;
  }
  if (draft.titleCustomId) {
    included["general.titleCustomId"] = draft.titleCustomId;
  }
  if (draft.titleOllamaModel) {
    included["general.titleOllamaModel"] = draft.titleOllamaModel;
  }
  if (draft.titleHostedBase) {
    included["general.titleHostedBase"] = draft.titleHostedBase;
  }
  if (draft.excludedBundleIds.trim()) {
    included["privacy.excludedBundleIds"] = draft.excludedBundleIds;
  }
  if (draft.appPolicies.trim()) {
    included["privacy.appPolicies"] = draft.appPolicies;
  }
  const excludedKeys: string[] = [];
  for (const [key, value] of Object.entries(extraRaw)) {
    if (!settingsKeyExportable(key) || valueLooksLikeMachinePath(value)) {
      excludedKeys.push(key);
      continue;
    }
    included[key] = value;
  }
  excludedKeys.sort();
  const sensitiveLiteralKeys: string[] = [];
  if (draft.excludedBundleIds.trim()) {
    sensitiveLiteralKeys.push("privacy.excludedBundleIds");
  }
  if (draft.appPolicies.trim()) {
    sensitiveLiteralKeys.push("privacy.appPolicies");
  }
  if (draft.standardChordEnabled) {
    sensitiveLiteralKeys.push("capture.standardChord");
  }
  return { included, sensitiveLiteralKeys, excludedKeys };
}
