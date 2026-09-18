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
  assert.doesNotMatch(html, /value="off"/);
  assert.doesNotMatch(html, /value="manual"/);
  assert.match(html, /data-export-preview/);
  assert.doesNotMatch(html, /permissionToken|installIdentity|\/Users\//);
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
});
