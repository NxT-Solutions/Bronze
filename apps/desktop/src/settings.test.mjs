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
    en["settings.field.excludedBundleIds.info"],
    "**Selections** in listed apps never enter the queue.",
  );
  assert.equal(
    en["settings.field.excludedBundleIds.infoWhen"],
    "**Use this** for password managers, banks, or any app you do not want captured.",
  );
  assert.equal(
    en["settings.field.excludedBundleIds.help"],
    "Bronze will not capture selections from these apps.",
  );
  assert.equal(
    en["settings.export.preview.info"],
    "**Export or import** a settings file on this Mac.",
  );
  assert.equal(
    en["settings.export.preview.infoList"],
    "**The preview** lists what the file includes before you save.",
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
    />\s*<strong>Starts Bronze<\/strong> when you log in to this Mac\.\s*</,
  );
  assert.doesNotMatch(html, /settings\.field\.launchAtLogin\.helpUntil/);
  assert.equal(en["settings.field.launchAtLogin"], "Launch at login");
  assert.equal(
    en["settings.field.launchAtLogin.help"],
    "**Starts Bronze** when you log in to this Mac.",
  );
  assert.equal(
    en["settings.field.launchAtLogin.helpUntil"],
    "**Off** until you turn it on.",
  );
  assert.equal(
    en["settings.field.launchAtLogin.infoName"],
    "About Launch at login",
  );
  assert.equal(
    en["settings.field.launchAtLogin.info"],
    "**Opens Bronze** when you log in to this Mac.",
  );
  assert.equal(
    en["settings.field.launchAtLogin.infoApproval"],
    "**Login Items** may still ask for approval.",
  );
  assert.equal(
    en["settings.field.launchAtLogin.status.requiresApproval"],
    "Allow Bronze in Login Items to finish.",
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
  assert.equal(
    en["settings.field.reduceMotion.info"],
    "**Follow this Mac** uses the system setting.",
  );
  assert.equal(
    en["settings.field.reduceMotion.infoAlways"],
    "**Always reduce** skips motion.",
  );
  assert.equal(
    en["settings.field.reduceMotion.infoPlay"],
    "**Play animations** keeps it on.",
  );
  assert.match(
    html,
    />\s*<strong>Follow this Mac<\/strong> uses the system setting\.\s*</,
  );
  assert.match(html, />\s*<strong>Always reduce<\/strong> skips motion\.\s*</);
  assert.match(html, />\s*<strong>Play animations<\/strong> keeps it on\.\s*</);
  assert.equal(en["settings.field.reduceMotion.system"], "Follow this Mac");
  assert.equal(en["settings.field.reduceMotion.on"], "Always reduce");
  assert.equal(en["settings.field.reduceMotion.off"], "Play animations");
  assert.match(html, /<option\s+value="off"/);
  assert.match(html, /data-settings-group="titles"/);
  assert.match(html, /data-title-engine-local/);
  assert.match(html, /data-reset-field="general.titleModel"/);
  assert.match(html, /data-i18n="settings.field.titleModel.local"/);
  assert.match(html, /data-i18n="settings.field.titleModel.integrations"/);
  assert.match(html, />\s*Title engines\s*</);
  assert.match(html, />\s*On this Mac\s*</);
  assert.match(html, />\s*Integrations\s*</);
  assert.equal(en["settings.group.titles"], "Title engines");
  assert.equal(en["settings.field.titleModel.local"], "On this Mac");
  assert.equal(en["settings.field.titleModel.integrations"], "Integrations");
  assert.equal(en["settings.field.titleModel.integrations.none"], "Off");
  assert.equal(
    en["settings.group.general.info"],
    "Launch at login, language, motion, and version on this Mac.",
  );
  assert.match(html, /data-app-version/);
  assert.match(html, /data-check-update/);
  assert.match(html, /id="update-sheet"/);
  assert.match(html, /data-i18n="settings.field.version.check"/);
  assert.equal(en["settings.field.version"], "Version");
  assert.equal(en["settings.field.version.check"], "Check for updates");
  assert.equal(
    en["settings.field.version.available"],
    "Version {version} is available.",
  );
  assert.equal(en["settings.field.version.brew"], "Copy Homebrew command");
  assert.match(
    html,
    />\s*<strong>Stays on this Mac<\/strong> and works offline\.\s*</,
  );
  assert.match(html, />\s*<strong>Extractive<\/strong> uses no model\.\s*</);
  assert.equal(en["settings.field.titleModel"], "Title engine");
  assert.equal(
    en["settings.field.titleModel.help"],
    "**Stays on this Mac** and works offline.",
  );
  assert.equal(
    en["settings.field.titleModel.helpExtractive"],
    "**Extractive** uses no model.",
  );
  assert.equal(en["settings.field.titleModel.infoName"], "About Title engine");
  assert.equal(
    en["settings.field.titleModel.info"],
    "**Stays on this Mac** and never sends text off this device.",
  );
  assert.equal(
    en["settings.field.titleModel.infoExtractive"],
    "**Extractive** uses no model.",
  );
  assert.match(html, /class="setting-info"/);
  assert.equal((html.match(/class="setting-info"/g) || []).length, 15);
  assert.equal((html.match(/class="setting-info-mark"/g) || []).length, 15);
  assert.match(html, /settings\.shortcuts\.title\.infoRestore/);
  assert.match(html, /data-i18n="settings.field.titleModel.info"/);
  assert.match(html, /data-i18n="settings.field.titleModel.infoExtractive"/);
  assert.match(html, /data-i18n="settings.field.reduceMotion.infoAlways"/);
  assert.match(html, /data-i18n="settings.field.reduceMotion.infoPlay"/);
  assert.match(html, /data-i18n="settings.field.locale.infoMenus"/);
  assert.match(html, /data-i18n="settings.field.excludedBundleIds.infoWhen"/);
  assert.match(html, /data-i18n="settings.export.preview.infoList"/);
  assert.match(html, /data-i18n="settings.field.titleModel.helpExtractive"/);
  assert.match(
    html,
    /aria-describedby="title-model-help title-model-help-extractive title-model-help-size title-model-status"/,
  );
  assert.match(html, /data-i18n-aria-label="settings.field.locale.infoName"/);
  assert.equal(
    en["settings.field.titleModel.extractive"],
    "Extractive — no extra memory",
  );
  assert.equal(
    en["settings.field.titleModel.smol135"],
    "SmolLM2 135M — about 105 MB of RAM, 2 CPU threads",
  );
  assert.equal(
    en["settings.field.titleModel.smol360"],
    "SmolLM2 360M — about 270 MB of RAM, 2 CPU threads",
  );
  assert.equal(
    en["settings.field.titleModel.qwen05"],
    "Qwen2.5 0.5B — about 490 MB of RAM, 2 CPU threads",
  );
  assert.equal(
    en["settings.field.titleModel.helpUse"],
    "**Holds that much memory** and uses 2 CPU threads while it writes a title.",
  );
  assert.match(html, /settings\.field\.titleModel\.helpUse/);
  assert.equal(
    en["settings.field.titleModel.missing"],
    "Vendored file missing — titles stay extractive until you run {command}.",
  );
  assert.equal(en["settings.field.titleModel.loading"], "Loading {engine}…");
  assert.equal(en["settings.field.titleModel.hashing"], "Checking {engine}…");
  assert.equal(
    en["settings.field.titleModel.loaded"],
    "Loaded — will title the next capture",
  );
  assert.match(html, /data-title-model-spinner/);
  assert.match(html, /aria-live="polite"/);
  assert.equal(
    en["settings.field.titleModel.custom"],
    "Imported GGUF — RAM follows the file, 2 CPU threads",
  );
  assert.equal(
    en["settings.field.titleModel.custom.sized"],
    "Imported GGUF — about {size} of RAM, 2 CPU threads",
  );
  assert.match(html, /Imported GGUF — RAM follows the file, 2 CPU threads/);
  assert.equal(en["settings.field.titleModel.ollama"], "Ollama on this Mac");
  assert.equal(
    en["settings.field.titleModel.infoHosted"],
    "**Sends truncated capture text** to {host}.",
  );
  assert.equal(
    en["settings.field.titleModel.hosted.keyHelp"],
    "Paste the key once — Bronze never shows it again.",
  );
  assert.match(html, /data-import-title-gguf/);
  assert.doesNotMatch(html, /class="btn-ghost"\s+data-import-title-gguf/);
  assert.match(html, /id="title-integration"/);
  assert.match(html, /id="title-hosted-key"/);
  assert.match(
    html,
    /data-title-engine-panel="hosted"[\s\S]*<div class="field-block">[\s\S]*id="title-hosted-base"/,
  );
  assert.match(
    html,
    /data-title-engine-panel="hosted"[\s\S]*<div class="field-block">[\s\S]*id="title-hosted-key"/,
  );
  assert.doesNotMatch(html, /Title:/);
  assert.doesNotMatch(html, /download/i);
});
