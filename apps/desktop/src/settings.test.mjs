import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "settings.html"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

test("settings window is searchable grouped with daily weekly backup and export preview", () => {
  assert.match(html, /id="settings"/);
  assert.match(html, /id="settings-search"/);
  assert.match(html, /class="chrome-search"/);
  assert.match(html, /class="chrome-search-icon"/);
  assert.match(html, /data-i18n-placeholder="settings.search.label"/);
  assert.match(html, /placeholder="Search settings"/);
  assert.doesNotMatch(html, /search-card/);
  assert.match(html, /data-settings-section/);
  assert.match(html, /data-settings-unit/);
  assert.match(html, /data-settings-search-empty/);
  assert.match(html, /data-settings-group="data"/);
  assert.match(html, /data-reset-field="data.backupSchedule"/);
  assert.match(html, /<option value="daily"/);
  assert.match(html, /<option value="weekly"/);
  assert.doesNotMatch(html, /id="backup-schedule"[\s\S]*?<option value="off"/);
  assert.doesNotMatch(html, /value="manual"/);
  assert.match(html, /data-export-preview/);
  assert.match(html, /data-export-included/);
  assert.match(html, /data-export-sensitive-list/);
  assert.match(html, /data-export-settings/);
  assert.match(html, /data-import-settings/);
  assert.match(html, /class="btn-primary"/);
  assert.match(html, /data-i18n="settings.export.action"/);
  assert.match(html, /data-i18n="settings.import.action"/);
  assert.match(html, />\s*Export\s*</);
  assert.match(html, />\s*Import\s*</);
  assert.match(html, /Included in this file/);
  assert.equal(en["settings.export.action"], "Export");
  assert.equal(en["settings.import.action"], "Import");
  assert.equal(en["settings.export.included"], "Included in this file");
  assert.equal(en["settings.export.preview"], "Export preview");
  assert.equal(
    en["settings.export.sensitive"],
    "User-entered literals may be sensitive",
  );
  assert.equal(en["settings.export.done"], "Exported");
  assert.equal(en["settings.import.done"], "Imported");
  assert.equal(
    en["settings.export.emptySensitive"],
    "No extra user-entered literals in this file.",
  );
  assert.equal(en["settings.export.key.excludedApps"], "Excluded apps");
  assert.equal(en["settings.export.key.appPolicies"], "App policies");
  assert.equal(en["settings.export.key.customShortcut"], "Custom shortcut");
  assert.equal(en["settings.export.key.profileLiterals"], "Profile literals");
  assert.match(html, /class="btn-ghost"\s+data-import-settings/);
  assert.match(html, /data-export-status/);
  assert.doesNotMatch(html, /<output data-export-preview>/);
  assert.doesNotMatch(html, /permissionToken|installIdentity|\/Users\//);
  assert.equal((html.match(/class="btn-primary"/g) || []).length, 1);
  assert.equal(en["settings.title"], "Settings");
  assert.equal(en["settings.backup.daily"], "Daily");
  assert.equal(en["settings.backup.weekly"], "Weekly");
  assert.match(html, /settings-live\.mjs/);
  assert.match(html, /<html lang="en">/);
  assert.match(html, /data-open-window="help"/);
  assert.doesNotMatch(html, /href="help\.html"/);
  assert.match(html, /id="excluded-apps-search"/);
  assert.match(html, /role="combobox"/);
  assert.doesNotMatch(html, /name="excludedBundleIds"/);
  assert.equal(
    en["settings.field.excludedBundleIds.help"],
    "Bronze will not capture selections from these apps.",
  );
  assert.equal(
    en["settings.field.excludedBundleIds.howto"],
    "Search to add many apps, or choose a missing app from Finder.",
  );
  assert.equal(en["settings.field.excludedBundleIds.choose"], "Choose app…");
  assert.match(html, /settings.field.excludedBundleIds.howto/);
  assert.match(html, /data-pick-installed-app/);
  assert.match(html, /class="chrome-search app-picker-search"/);
  assert.match(html, /data-search-clear/);
  assert.match(html, /data-i18n-aria-label="settings.search.clear"/);
  assert.equal(en["settings.search.clear"], "Clear search");
  assert.match(html, /id="title-model"/);
  assert.match(html, /id="launch-at-login"/);
  assert.match(html, /type="checkbox"/);
  assert.match(html, /name="launchAtLogin"/);
  assert.match(html, /data-reset-field="general.launchAtLogin"/);
  assert.match(html, /data-i18n="settings.field.launchAtLogin"/);
  assert.match(html, /data-login-item-status/);
  assert.match(
    html,
    />\s*Starts Bronze when you log in to this Mac; off until you turn it on\.\s*</,
  );
  assert.equal(en["settings.field.launchAtLogin"], "Launch at login");
  assert.equal(
    en["settings.field.launchAtLogin.help"],
    "Starts Bronze when you log in to this Mac; off until you turn it on.",
  );
  assert.equal(
    en["settings.field.launchAtLogin.infoName"],
    "About Launch at login",
  );
  assert.equal(
    en["settings.field.launchAtLogin.info"],
    "Saves whether Bronze should open when this Mac starts; Login Items may still ask for approval, and a debug build that is not an app cannot register.",
  );
  assert.equal(
    en["settings.field.launchAtLogin.status.unavailable"],
    "This debug build cannot register as a login item.",
  );
  assert.match(html, /id="reduce-motion"/);
  assert.match(html, /data-reset-field="general.reduceMotion"/);
  assert.match(html, /data-i18n="settings.field.reduceMotion"/);
  assert.match(html, />\s*Follows this Mac unless you override it\.\s*</);
  assert.equal(en["settings.field.reduceMotion"], "Reduce motion");
  assert.equal(
    en["settings.field.reduceMotion.help"],
    "Follows this Mac unless you override it.",
  );
  assert.equal(
    en["settings.field.reduceMotion.infoName"],
    "About Reduce motion",
  );
  assert.equal(en["settings.field.reduceMotion.system"], "Follow this Mac");
  assert.equal(en["settings.field.reduceMotion.on"], "Always reduce");
  assert.equal(en["settings.field.reduceMotion.off"], "Play animations");
  assert.match(html, /<option\s+value="off"/);
  assert.match(html, /data-reset-field="general.titleModel"/);
  assert.match(html, /data-i18n="settings.field.titleModel"/);
  assert.match(html, />\s*Title engine\s*</);
  assert.match(
    html,
    />\s*A local model bundled in Bronze that never leaves this Mac and works offline; extractive uses no model\.\s*</,
  );
  assert.equal(en["settings.field.titleModel"], "Title engine");
  assert.equal(
    en["settings.field.titleModel.help"],
    "A local model bundled in Bronze that never leaves this Mac and works offline; extractive uses no model.",
  );
  assert.equal(en["settings.field.titleModel.infoName"], "About Title engine");
  assert.match(html, /class="setting-info"/);
  assert.equal((html.match(/class="setting-info"/g) || []).length, 12);
  assert.equal((html.match(/class="setting-info-mark"/g) || []).length, 12);
  assert.match(html, /data-i18n="settings.field.titleModel.info"/);
  assert.match(html, /data-i18n-aria-label="settings.field.locale.infoName"/);
  assert.equal(
    en["settings.field.titleModel.extractive"],
    "Extractive — no model file",
  );
  assert.equal(
    en["settings.field.titleModel.smol135"],
    "SmolLM2 135M — about 105 MB, 8 GB RAM",
  );
  assert.equal(
    en["settings.field.titleModel.smol360"],
    "SmolLM2 360M — about 271 MB, 16 GB RAM",
  );
  assert.equal(
    en["settings.field.titleModel.qwen05"],
    "Qwen2.5 0.5B — about 491 MB, 32 GB RAM",
  );
  assert.equal(
    en["settings.field.titleModel.missing"],
    "Vendored file missing; keep extractive titles and run {command}",
  );
  assert.equal(en["settings.field.titleModel.loading"], "Loading {engine}…");
  assert.equal(en["settings.field.titleModel.hashing"], "Checking {engine}…");
  assert.equal(
    en["settings.field.titleModel.loaded"],
    "Loaded — will title the next capture",
  );
  assert.match(html, /data-title-model-spinner/);
  assert.match(html, /aria-live="polite"/);
  assert.doesNotMatch(html, /Title:/);
  assert.doesNotMatch(html, /download/i);
});
