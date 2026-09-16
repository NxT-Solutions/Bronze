import { describe, expect, it } from "vitest";
import {
  defaultSettingsDraft,
  parseBackupSchedule,
  previewSettingsExport,
  resetSettingsField,
  searchSettingsFields,
  settingsKeyExportable,
} from "./settings-schema";

describe("settings schema (SET-001)", () => {
  it("search finds backup and rejects off schedule", () => {
    const hits = searchSettingsFields("backup");
    expect(hits.some((field) => field.id === "data.backupSchedule")).toBe(true);
    expect(parseBackupSchedule("daily")).toBe("daily");
    expect(parseBackupSchedule("weekly")).toBe("weekly");
    expect(parseBackupSchedule("off")).toBeNull();
    expect(parseBackupSchedule("manual")).toBeNull();
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
