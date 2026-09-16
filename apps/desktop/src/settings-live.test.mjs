import assert from "node:assert/strict";
import { test } from "node:test";
import { applySettingsForm, patchSettingsFromForm } from "./settings-live.mjs";

test("settings form patches backup schedule and excluded apps", () => {
  const settings = {
    data: { backupSchedule: "daily" },
    privacy: { excludedBundleIds: ["com.example"] },
  };
  const schedule = { value: "weekly" };
  const excluded = { value: "com.one, com.two" };
  const root = {
    querySelector(sel) {
      if (sel === "#backup-schedule") return schedule;
      if (sel === "#excluded-bundle-ids") return excluded;
      return null;
    },
  };
  applySettingsForm(root, settings);
  assert.equal(schedule.value, "daily");
  assert.equal(excluded.value, "com.example");
  schedule.value = "weekly";
  excluded.value = "com.one, com.two";
  const next = patchSettingsFromForm(settings, root);
  assert.equal(next.data.backupSchedule, "weekly");
  assert.deepEqual(next.privacy.excludedBundleIds, ["com.one", "com.two"]);
});
