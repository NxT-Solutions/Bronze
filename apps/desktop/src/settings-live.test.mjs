import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applySettingsForm,
  applySettingsSearch,
  filterInstalledApps,
  isSafeBundleId,
  isSafeDisplayName,
  patchSettingsFromForm,
  pickExcludedApp,
  pickerFailureCode,
  readExcludedBundleIds,
  resolveExcludedApps,
  settingsSearchNeedle,
  settingsUnitHaystack,
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
  const excluded = { dataset: { excludedIds: "" } };
  const locale = { value: "en" };
  const root = {
    querySelector(sel) {
      if (sel === "#backup-schedule") return schedule;
      if (sel === "#excluded-apps") return excluded;
      if (sel === "#ui-locale") return locale;
      return null;
    },
  };
  applySettingsForm(root, settings);
  assert.equal(schedule.value, "daily");
  assert.equal(excluded.dataset.excludedIds, "com.example");
  assert.equal(locale.value, "en");
  settings.general.locale = "nl";
  applySettingsForm(root, settings);
  assert.equal(locale.value, "nl");
  schedule.value = "weekly";
  excluded.dataset.excludedIds = "com.one\ncom.two";
  locale.value = "fr";
  const next = patchSettingsFromForm(settings, root);
  assert.equal(next.data.backupSchedule, "weekly");
  assert.deepEqual(next.privacy.excludedBundleIds, ["com.one", "com.two"]);
  assert.equal(next.general.locale, "fr");
  assert.equal(switcherLocale("system"), "en");
  assert.equal(switcherLocale("de"), "de");
});

test("excluded app picker searches installed apps and keeps many ids", () => {
  assert.equal(isSafeBundleId("com.apple.Safari"), true);
  assert.equal(isSafeBundleId("/Applications/Safari.app"), false);
  const catalog = [
    { bundleId: "com.apple.Safari", name: "Safari" },
    { bundleId: "com.apple.TextEdit", name: "TextEdit" },
    { bundleId: "/Applications/Evil.app", name: "Evil" },
  ];
  assert.deepEqual(
    filterInstalledApps(catalog, "saf", ["com.apple.TextEdit"]).map(
      (app) => app.bundleId,
    ),
    ["com.apple.Safari"],
  );
  assert.deepEqual(
    resolveExcludedApps(["com.apple.Safari", "com.missing.app"], catalog),
    [
      { bundleId: "com.apple.Safari", name: "Safari" },
      { bundleId: "com.missing.app", name: "com.missing.app" },
    ],
  );
  assert.match(html, /id="excluded-apps-search"/);
  assert.match(html, /role="combobox"/);
  assert.match(html, /settings.field.excludedBundleIds.help/);
  assert.doesNotMatch(html, /name="excludedBundleIds"/);
  assert.doesNotMatch(html, /id="excluded-bundle-ids"/);
  assert.match(live, /bindShortcutRegistry/);
  assert.match(live, /list_installed_apps/);
  assert.match(live, /pick_installed_app/);
  assert.match(live, /app_icon_data_url/);
  assert.match(live, /btn-icon/);
  assert.match(live, /app-picker-option-icon/);
  assert.match(live, /bindSearchClear/);
  assert.match(live, /is-ready/);
  assert.match(live, /data-i18n-aria-label/);
  assert.doesNotMatch(live, /remove\.textContent = removeLabel/);
  assert.match(html, /data-pick-installed-app/);
  assert.match(html, /settings.field.excludedBundleIds.howto/);
  assert.match(html, /settings.field.excludedBundleIds.choose/);
  assert.doesNotMatch(live, /join\(", "\)/);
  assert.doesNotMatch(live, /pick_installed_app\([^)]*path/);
  assert.equal(isSafeDisplayName("Safari"), true);
  assert.equal(isSafeDisplayName("/Applications/Safari.app"), false);
  assert.equal(pickerFailureCode("picker_cancelled"), "picker_cancelled");
  assert.equal(pickerFailureCode("picker_unavailable"), "picker_unavailable");
  assert.equal(pickerFailureCode("nope"), "invalid_app");
});

test("finder pick adds a bundle without sending a path", async () => {
  const host = {
    dataset: { excludedIds: "" },
    _installedApps: [],
    querySelector() {
      return null;
    },
  };
  const picked = await pickExcludedApp(host, async (cmd, args) => {
    assert.equal(cmd, "pick_installed_app");
    assert.equal(args, undefined);
    return { bundleId: "com.setapp.Foo", name: "Foo" };
  });
  assert.equal(picked, "picked");
  assert.deepEqual(readExcludedBundleIds(host), ["com.setapp.Foo"]);
  const cancelled = await pickExcludedApp(host, async () => {
    throw "picker_cancelled";
  });
  assert.equal(cancelled, "picker_cancelled");
  assert.deepEqual(readExcludedBundleIds(host), ["com.setapp.Foo"]);
  const bad = await pickExcludedApp(host, async () => {
    return { bundleId: "/Applications/Evil.app", name: "Evil" };
  });
  assert.equal(bad, "invalid_app");
  assert.deepEqual(readExcludedBundleIds(host), ["com.setapp.Foo"]);
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
  assert.match(live, /applySettingsSearch/);
  assert.doesNotMatch(live, /search_settings_fields/);
});

function fakeNode(spec = {}) {
  const node = {
    hidden: false,
    textContent: spec.text ?? "",
    children: spec.children ?? [],
    _attr: spec.attr ?? {},
    getAttribute(name) {
      return Object.hasOwn(node._attr, name) ? node._attr[name] : null;
    },
    hasAttribute(name) {
      return Object.hasOwn(node._attr, name);
    },
    querySelector(sel) {
      return node.querySelectorAll(sel)[0] ?? null;
    },
    querySelectorAll(sel) {
      const out = [];
      for (const child of node.children) {
        if (fakeMatches(child, sel)) {
          out.push(child);
        }
        out.push(...child.querySelectorAll(sel));
      }
      return out;
    },
  };
  return node;
}

function fakeMatches(node, sel) {
  if (sel === "label") {
    return node._attr.tag === "label";
  }
  if (sel === "option") {
    return node._attr.tag === "option";
  }
  if (sel === "input, select, textarea") {
    return ["input", "select", "textarea"].includes(node._attr.tag);
  }
  if (sel.startsWith("[") && sel.endsWith("]")) {
    return Object.hasOwn(node._attr, sel.slice(1, -1));
  }
  return false;
}

test("settings search matches visible labels and not reset chrome", () => {
  assert.equal(settingsSearchNeedle("  Backup  "), "backup");
  const language = fakeNode({
    attr: { "data-settings-unit": "" },
    text: "Language English Reset field",
    children: [
      fakeNode({ attr: { tag: "label" }, text: "Language" }),
      fakeNode({
        attr: { tag: "select", name: "locale" },
        text: "",
      }),
      fakeNode({ attr: { tag: "option" }, text: "English" }),
    ],
  });
  language.querySelector("input, select, textarea").value = "en";
  language.querySelector("input, select, textarea").getAttribute = (name) =>
    name === "name" ? "locale" : null;
  const hay = settingsUnitHaystack(language);
  assert.match(hay, /Language/);
  assert.match(hay, /English/);
  assert.doesNotMatch(hay, /Reset field/);

  const general = fakeNode({
    attr: { "data-settings-section": "" },
    children: [
      fakeNode({ attr: { "data-settings-title": "" }, text: "General" }),
      language,
    ],
  });
  const dataUnit = fakeNode({
    attr: { "data-settings-unit": "" },
    text: "Backup schedule Daily Reset field",
    children: [
      fakeNode({ attr: { tag: "label" }, text: "Backup schedule" }),
      fakeNode({ attr: { tag: "option" }, text: "Daily" }),
    ],
  });
  const data = fakeNode({
    attr: { "data-settings-section": "" },
    children: [
      fakeNode({ attr: { "data-settings-title": "" }, text: "Data" }),
      dataUnit,
    ],
  });
  const shortcuts = fakeNode({
    attr: { "data-settings-section": "" },
    children: [
      fakeNode({ attr: { "data-settings-title": "" }, text: "Shortcuts" }),
      fakeNode({
        attr: { "data-settings-unit": "" },
        text: "Capture selection Shift double-tap",
      }),
    ],
  });
  const form = fakeNode({
    attr: { "data-settings-form": "" },
    children: [general, data],
  });
  const empty = fakeNode({ attr: { "data-settings-search-empty": "" } });
  empty.hidden = true;
  const root = fakeNode({
    children: [form, shortcuts, empty],
  });

  applySettingsSearch(root, "language");
  assert.equal(general.hidden, false);
  assert.equal(language.hidden, false);
  assert.equal(data.hidden, true);
  assert.equal(shortcuts.hidden, true);
  assert.equal(form.hidden, false);
  assert.equal(empty.hidden, true);

  applySettingsSearch(root, "shortcut");
  assert.equal(general.hidden, true);
  assert.equal(data.hidden, true);
  assert.equal(shortcuts.hidden, false);
  assert.equal(form.hidden, true);
  assert.equal(empty.hidden, true);

  applySettingsSearch(root, "zzzz");
  assert.equal(form.hidden, true);
  assert.equal(shortcuts.hidden, true);
  assert.equal(empty.hidden, false);

  applySettingsSearch(root, "");
  assert.equal(general.hidden, false);
  assert.equal(data.hidden, false);
  assert.equal(shortcuts.hidden, false);
  assert.equal(form.hidden, false);
  assert.equal(empty.hidden, true);
});
