import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applySettingsForm,
  applySettingsSearch,
  applyTitleModelStatus,
  exportCategoryLabel,
  exportSensitiveLabel,
  filterInstalledApps,
  formatTitleModelStatus,
  isSafeBundleId,
  isSafeDisplayName,
  parseTitleModelId,
  patchSettingsFromForm,
  pickExcludedApp,
  pickerFailureCode,
  readExcludedBundleIds,
  refreshTitleModelStatus,
  renderExportPreview,
  resolveExcludedApps,
  settingsExportFailureCode,
  settingsSearchNeedle,
  settingsUnitHaystack,
  switcherLocale,
  TITLE_MODEL_IDS,
} from "./settings-live.mjs";

const rootDir = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(rootDir, "settings.html"), "utf8");
const live = readFileSync(join(rootDir, "settings-live.mjs"), "utf8");

test("settings form patches backup schedule, excluded apps, and locale", () => {
  const settings = {
    general: { locale: "system", titleModel: "smol-360" },
    data: { backupSchedule: "daily" },
    privacy: { excludedBundleIds: ["com.example"] },
  };
  const schedule = { value: "weekly" };
  const excluded = { dataset: { excludedIds: "" } };
  const locale = { value: "en" };
  const titleModel = { value: "extractive" };
  const root = {
    querySelector(sel) {
      if (sel === "#backup-schedule") return schedule;
      if (sel === "#excluded-apps") return excluded;
      if (sel === "#ui-locale") return locale;
      if (sel === "#title-model") return titleModel;
      return null;
    },
  };
  applySettingsForm(root, settings);
  assert.equal(schedule.value, "daily");
  assert.equal(excluded.dataset.excludedIds, "com.example");
  assert.equal(locale.value, "en");
  assert.equal(titleModel.value, "smol-360");
  settings.general.locale = "nl";
  applySettingsForm(root, settings);
  assert.equal(locale.value, "nl");
  schedule.value = "weekly";
  excluded.dataset.excludedIds = "com.one\ncom.two";
  locale.value = "fr";
  titleModel.value = "qwen-05";
  const next = patchSettingsFromForm(settings, root);
  assert.equal(next.data.backupSchedule, "weekly");
  assert.deepEqual(next.privacy.excludedBundleIds, ["com.one", "com.two"]);
  assert.equal(next.general.locale, "fr");
  assert.equal(next.general.titleModel, "qwen-05");
  assert.equal(switcherLocale("system"), "en");
  assert.equal(switcherLocale("de"), "de");
  const preserved = patchSettingsFromForm(
    { general: { titleModel: "smol-135" }, data: {}, privacy: {} },
    {
      querySelector() {
        return null;
      },
    },
  );
  assert.equal(preserved.general.titleModel, "smol-135");
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
  assert.match(live, /preview_settings_export/);
  assert.match(live, /export_settings_file/);
  assert.match(live, /import_settings_file/);
  assert.match(live, /requestedPath: null/);
  assert.doesNotMatch(live, /export_library_archive/);
  assert.doesNotMatch(live, /import_library_archive/);
  assert.equal(settingsExportFailureCode("picker_cancelled"), "");
  assert.equal(
    settingsExportFailureCode("picker_unavailable"),
    "picker_unavailable",
  );
  assert.equal(
    settingsExportFailureCode(new Error("settings_wrong_format")),
    "settings_wrong_format",
  );
  assert.equal(settingsExportFailureCode("nope"), "settings_invalid");
  assert.equal(exportCategoryLabel("shortcuts"), "Shortcuts");
  assert.equal(exportCategoryLabel("profiles"), "Profiles");
  assert.equal(exportCategoryLabel("privacy"), "Privacy");
  assert.equal(
    exportSensitiveLabel("privacy.excludedBundleIds"),
    "Excluded apps",
  );
  assert.equal(
    exportSensitiveLabel("shortcuts.queue.search"),
    "Custom shortcut",
  );
  assert.equal(
    exportSensitiveLabel("profiles.custom-prompt.name"),
    "Profile literals",
  );
});

test("export preview lists included categories and sensitive leftovers", () => {
  const included = {
    children: [],
    replaceChildren(...nodes) {
      this.children = nodes;
    },
    append(node) {
      this.children.push(node);
    },
  };
  const sensitive = {
    children: [],
    replaceChildren(...nodes) {
      this.children = nodes;
    },
    append(node) {
      this.children.push(node);
    },
  };
  const root = {
    querySelector(sel) {
      if (sel === "[data-export-included]") {
        return included;
      }
      if (sel === "[data-export-sensitive-list]") {
        return sensitive;
      }
      return null;
    },
  };
  const prevDoc = globalThis.document;
  globalThis.document = {
    createElement() {
      return { textContent: "" };
    },
  };
  try {
    renderExportPreview(root, {
      includedCategories: ["privacy", "shortcuts", "profiles"],
      includedFields: ["privacy.excludedBundleIds", "shortcuts"],
      sensitiveLiteralKeys: [
        "privacy.excludedBundleIds",
        "shortcuts.queue.search",
        "profiles.custom-prompt.name",
      ],
    });
    assert.deepEqual(
      included.children.map((node) => node.textContent),
      ["Privacy", "Shortcuts", "Profiles"],
    );
    assert.deepEqual(
      sensitive.children.map((node) => node.textContent),
      ["Excluded apps", "Custom shortcut", "Profile literals"],
    );
    renderExportPreview(root, {
      includedCategories: ["data"],
      sensitiveLiteralKeys: [],
    });
    assert.deepEqual(
      included.children.map((node) => node.textContent),
      ["Data"],
    );
    assert.deepEqual(
      sensitive.children.map((node) => node.textContent),
      ["No extra user-entered literals in this file."],
    );
  } finally {
    globalThis.document = prevDoc;
  }
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

test("title model status shows present vs vendor command and never fetches", async () => {
  assert.deepEqual(
    [...TITLE_MODEL_IDS],
    ["extractive", "smol-135", "smol-360", "qwen-05"],
  );
  assert.equal(parseTitleModelId("qwen-05"), "qwen-05");
  assert.equal(parseTitleModelId("needle"), "");
  assert.equal(
    formatTitleModelStatus({ id: "extractive", present: true }),
    "Extractive titles use no model file.",
  );
  assert.equal(
    formatTitleModelStatus({ id: "smol-360", present: true }),
    "This file is on this Mac and can title the next capture.",
  );
  assert.equal(
    formatTitleModelStatus({
      id: "qwen-05",
      present: false,
      vendorCommand: "sh bronze-title-model/scripts/vendor-gguf.sh qwen-05",
    }),
    "Vendored file missing; keep extractive titles and run sh bronze-title-model/scripts/vendor-gguf.sh qwen-05",
  );
  assert.equal(
    formatTitleModelStatus(
      { id: "qwen-05", present: false },
      { unavailable: true },
    ),
    "Title engines could not be listed.",
  );
  const status = { textContent: "" };
  const select = { value: "qwen-05" };
  const root = {
    querySelector(sel) {
      if (sel === "[data-title-model-status]") return status;
      if (sel === "#title-model") return select;
      return null;
    },
  };
  applyTitleModelStatus(root, {
    id: "qwen-05",
    present: false,
    vendorCommand: "sh bronze-title-model/scripts/vendor-gguf.sh qwen-05",
  });
  assert.match(status.textContent, /vendor-gguf\.sh qwen-05/);
  assert.doesNotMatch(status.textContent, /Title:/);
  const listed = await refreshTitleModelStatus(root, async (cmd) => {
    assert.equal(cmd, "list_title_models");
    return [
      { id: "extractive", present: true, vendorCommand: "" },
      {
        id: "qwen-05",
        present: false,
        vendorCommand: "sh bronze-title-model/scripts/vendor-gguf.sh qwen-05",
      },
    ];
  });
  assert.equal(listed?.[1]?.present, false);
  assert.match(status.textContent, /Vendored file missing/);
  assert.match(html, /id="title-model"/);
  assert.match(html, /name="titleModel"/);
  assert.match(html, /data-reset-field="general.titleModel"/);
  assert.match(html, /data-title-model-status/);
  assert.match(html, /role="status"/);
  assert.match(html, /value="extractive"/);
  assert.match(html, /value="smol-135"/);
  assert.match(html, /value="smol-360"/);
  assert.match(html, /value="qwen-05"/);
  assert.doesNotMatch(html, /download/i);
  assert.doesNotMatch(html, /huggingface/i);
  assert.match(live, /list_title_models/);
  assert.match(live, /#title-model/);
  assert.doesNotMatch(live, /huggingface/i);
  assert.doesNotMatch(live, /https:\/\//);
});
