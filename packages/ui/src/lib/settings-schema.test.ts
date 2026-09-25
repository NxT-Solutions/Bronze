import { describe, expect, it } from "vitest";
import {
  defaultSettingsDraft,
  parseBackupSchedule,
  parseTitleModelId,
  previewSettingsExport,
  resetSettingsField,
  searchSettingsFields,
  settingsKeyExportable,
} from "./settings-schema";

describe("settings schema (SET-001)", () => {
  it("defaults the standard chord to enabled", () => {
    expect(defaultSettingsDraft().standardChordEnabled).toBe(true);
  });

  it("search finds backup and rejects off schedule", () => {
    const hits = searchSettingsFields("backup");
    expect(hits.some((field) => field.id === "data.backupSchedule")).toBe(true);
    expect(hits[0]?.id).toBe("data.backupSchedule");
    expect(
      searchSettingsFields("BCKP").some(
        (field) => field.id === "data.backupSchedule",
      ),
    ).toBe(true);
    expect(
      searchSettingsFields("BACKUP").some(
        (field) => field.id === "data.backupSchedule",
      ),
    ).toBe(true);
    expect(searchSettingsFields("zzzz")).toEqual([]);
    expect(searchSettingsFields("  ").map((field) => field.id)).toEqual(
      searchSettingsFields("").map((field) => field.id),
    );
    expect(parseBackupSchedule("daily")).toBe("daily");
    expect(parseBackupSchedule("weekly")).toBe("weekly");
    expect(parseBackupSchedule("off")).toBeNull();
    expect(parseBackupSchedule("manual")).toBeNull();
  });

  it("title model is exportable, searchable, and reset does not invent a choice", () => {
    expect(parseTitleModelId("qwen-05")).toBe("qwen-05");
    expect(parseTitleModelId("smol-360")).toBe("smol-360");
    expect(parseTitleModelId("custom")).toBe("custom");
    expect(parseTitleModelId("ollama")).toBe("ollama");
    expect(parseTitleModelId("hosted-openai")).toBe("hosted-openai");
    expect(parseTitleModelId("needle")).toBeNull();
    expect(settingsKeyExportable("hostedKey")).toBe(false);
    expect(settingsKeyExportable("apiKey")).toBe(false);
    expect(
      searchSettingsFields("qwen").some(
        (field) => field.id === "general.titleModel",
      ),
    ).toBe(true);
    expect(settingsKeyExportable("general.titleModel")).toBe(true);
    const preview = previewSettingsExport({
      ...defaultSettingsDraft(),
      titleModel: "smol-360",
    });
    expect(preview.included["general.titleModel"]).toBe("smol-360");
    expect(preview.sensitiveLiteralKeys).not.toContain("general.titleModel");
    const customPreview = previewSettingsExport({
      ...defaultSettingsDraft(),
      titleModel: "custom",
      titleCustomId: "ab".repeat(32),
      titleCustomName: "tiny.gguf",
    });
    expect(customPreview.included["general.titleCustomId"]).toBe(
      "ab".repeat(32),
    );
    expect(JSON.stringify(customPreview.included)).not.toContain("/");
    const reset = resetSettingsField(
      { ...defaultSettingsDraft(), titleModel: "qwen-05" },
      "general.titleModel",
    );
    expect(reset.titleModel).toBe("");
  });

  it("reset field restores daily backup", () => {
    const draft = resetSettingsField(
      { ...defaultSettingsDraft(), backupSchedule: "weekly" },
      "data.backupSchedule",
    );
    expect(draft.backupSchedule).toBe("daily");
  });

  it("export preview flags sensitive literals and drops credentials tokens paths", () => {
    const preview = previewSettingsExport(
      {
        ...defaultSettingsDraft(),
        excludedBundleIds: "com.bank.app",
        appPolicies: "com.vault",
        standardChordEnabled: true,
      },
      {
        permissionToken: "abc",
        installIdentity: "machine-1",
        logPath: "/Users/me/bronze.log",
        appCredential: "secret",
        theme: "dark",
      },
    );
    expect(preview.included["data.backupSchedule"]).toBe("daily");
    expect(preview.included.theme).toBe("dark");
    expect(preview.included.permissionToken).toBeUndefined();
    expect(preview.included.logPath).toBeUndefined();
    expect(preview.sensitiveLiteralKeys).toContain("privacy.excludedBundleIds");
    expect(preview.sensitiveLiteralKeys).toContain("privacy.appPolicies");
    expect(preview.sensitiveLiteralKeys).toContain("capture.standardChord");
    expect(preview.excludedKeys).toContain("permissionToken");
    expect(settingsKeyExportable("permissionToken")).toBe(false);
    expect(JSON.stringify(preview.included)).not.toContain("abc");
    expect(JSON.stringify(preview.included)).not.toContain("/Users/me");
  });
});
