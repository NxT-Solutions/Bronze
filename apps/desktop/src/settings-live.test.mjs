import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applySettingsForm,
  patchSettingsFromForm,
  switcherLocale,
} from "./settings-live.mjs";

const rootDir = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(rootDir, "settings.html"), "utf8");
const live = readFileSync(join(rootDir, "settings-live.mjs"), "utf8");

test("settings form patches backup schedule, excluded apps, and locale", () => {
  const settings = {
    general: { locale: "system" },
    data: { backupSchedule: "daily" },
    privacy: { excludedBundleIds: ["com.example"] },
  };
  const schedule = { value: "weekly" };
  const excluded = { value: "com.one, com.two" };
  const locale = { value: "en" };
  const root = {
    querySelector(sel) {
      if (sel === "#backup-schedule") return schedule;
      if (sel === "#excluded-bundle-ids") return excluded;
      if (sel === "#ui-locale") return locale;
      return null;
    },
  };
  applySettingsForm(root, settings);
  assert.equal(schedule.value, "daily");
  assert.equal(excluded.value, "com.example");
  assert.equal(locale.value, "en");
  settings.general.locale = "nl";
  applySettingsForm(root, settings);
  assert.equal(locale.value, "nl");
  schedule.value = "weekly";
  excluded.value = "com.one, com.two";
  locale.value = "fr";
  const next = patchSettingsFromForm(settings, root);
  assert.equal(next.data.backupSchedule, "weekly");
  assert.deepEqual(next.privacy.excludedBundleIds, ["com.one", "com.two"]);
  assert.equal(next.general.locale, "fr");
  assert.equal(switcherLocale("system"), "en");
  assert.equal(switcherLocale("de"), "de");
});

test("settings language switcher uses endonyms and option lang", () => {
  assert.match(html, /data-settings-group="general"/);
  assert.match(html, /data-i18n="settings.field.locale"/);
  assert.match(
    html,
    /<option value="en" lang="en" selected>🇬🇧 English<\/option>/,
  );
  assert.match(html, /<option value="nl" lang="nl">🇳🇱 Nederlands<\/option>/);
  assert.match(html, /<option value="fr" lang="fr">🇫🇷 Français<\/option>/);
  assert.match(html, /<option value="de" lang="de">🇩🇪 Deutsch<\/option>/);
  assert.match(html, /<option value="es" lang="es">🇪🇸 Español<\/option>/);
  assert.match(html, /<option value="it" lang="it">🇮🇹 Italiano<\/option>/);
  assert.match(html, /data-reset-field="general.locale"/);
  assert.match(live, /emitUiLocaleChanged/);
});
