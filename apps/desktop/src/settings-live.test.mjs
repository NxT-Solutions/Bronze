import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parseQueueSort, queueSortChange } from "./queue-sort.mjs";
import {
  applyAppVersionInfo,
  applySettingsForm,
  applySettingsSearch,
  applyTitleEngineLifecycle,
  applyTitleModelStatus,
  applyUpdateCheck,
  bindSettingInfo,
  bindTitleEngineStatus,
  checkForAppUpdate,
  displayUpdateNote,
  exportCategoryLabel,
  exportSensitiveLabel,
  filterInstalledApps,
  formatCustomTitleOption,
  formatTitleEngineLifecycle,
  formatTitleFileSize,
  formatTitleModelStatus,
  isSafeBundleId,
  isSafeDisplayName,
  parseInstallSource,
  parseLoginItemStatus,
  parseTitleModelId,
  parseUpdateAction,
  patchSettingsFromForm,
  pickExcludedApp,
  pickerFailureCode,
  readExcludedBundleIds,
  refreshAppVersion,
  refreshLoginItemStatus,
  refreshTitleModelStatus,
  renderExportPreview,
  renderUpdateNotes,
  resolveExcludedApps,
  settingsExportFailureCode,
  settingsSearchMatches,
  settingsSearchNeedle,
  settingsUnitHaystack,
  switcherLocale,
  syncTitleEnginePanels,
  TITLE_ENGINE_STATUS_EVENT,
  TITLE_MODEL_IDS,
  titleEngineBusy,
} from "./settings-live.mjs";

const rootDir = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(rootDir, "settings.html"), "utf8");
const live = readFileSync(join(rootDir, "settings-live.mjs"), "utf8");

test("settings form keeps queue order on the copy group", () => {
  const queueSort = { value: "newest" };
  const root = {
    querySelector(sel) {
      if (sel === "#queue-sort") return queueSort;
      return null;
    },
  };
  const settings = {
    general: {},
    copy: {
      defaultProfileId: "plain",
      returnToPriorApp: true,
      queueSort: "oldest",
    },
    data: {},
    privacy: {},
  };
  applySettingsForm(root, settings);
  assert.equal(queueSort.value, "oldest");
  queueSort.value = "newest";
  const next = patchSettingsFromForm(settings, root);
  assert.equal(next.copy.queueSort, "newest");
  assert.equal(next.copy.defaultProfileId, "plain");
  assert.equal(next.copy.returnToPriorApp, true);
  assert.equal(parseQueueSort("oldest"), "oldest");
  assert.equal(parseQueueSort("rank"), "newest");
  assert.deepEqual(queueSortChange("newest", "oldest"), { sort: "oldest" });
  assert.equal(queueSortChange("oldest", "oldest"), null);
});

test("settings form patches backup schedule, excluded apps, and locale", () => {
  const settings = {
    general: {
      locale: "system",
      titleModel: "smol-360",
      reduceMotion: "system",
      launchAtLogin: false,
    },
    data: { backupSchedule: "daily" },
    privacy: { excludedBundleIds: ["com.example"] },
    accessibility: { motion: "system" },
  };
  const schedule = { value: "weekly" };
  const excluded = { dataset: { excludedIds: "" } };
  const locale = { value: "en" };
  const titleModel = { value: "extractive" };
  const titleIntegration = { value: "none" };
  const reduceMotion = { value: "system" };
  const launchAtLogin = { checked: false };
  const root = {
    querySelector(sel) {
      if (sel === "#backup-schedule") return schedule;
      if (sel === "#excluded-apps") return excluded;
      if (sel === "#ui-locale") return locale;
      if (sel === "#title-model") return titleModel;
      if (sel === "#title-integration") return titleIntegration;
      if (sel === "#reduce-motion") return reduceMotion;
      if (sel === "#launch-at-login") return launchAtLogin;
      return null;
    },
  };
  applySettingsForm(root, settings);
  assert.equal(schedule.value, "daily");
  assert.equal(excluded.dataset.excludedIds, "com.example");
  assert.equal(locale.value, "en");
  assert.equal(titleModel.value, "smol-360");
  assert.equal(titleIntegration.value, "none");
  settings.general.titleModel = "hosted-openai";
  applySettingsForm(root, settings);
  assert.equal(titleIntegration.value, "hosted-openai");
  settings.general.titleModel = "smol-360";
  applySettingsForm(root, settings);
  assert.equal(titleModel.value, "smol-360");
  assert.equal(titleIntegration.value, "none");
  assert.equal(reduceMotion.value, "system");
  assert.equal(launchAtLogin.checked, false);
  settings.general.locale = "nl";
  settings.general.reduceMotion = "off";
  settings.general.launchAtLogin = true;
  applySettingsForm(root, settings);
  assert.equal(locale.value, "nl");
  assert.equal(reduceMotion.value, "off");
  assert.equal(launchAtLogin.checked, true);
  schedule.value = "weekly";
  excluded.dataset.excludedIds = "com.one\ncom.two";
  locale.value = "fr";
  titleModel.value = "qwen-05";
  reduceMotion.value = "on";
  launchAtLogin.checked = false;
  const next = patchSettingsFromForm(settings, root);
  assert.equal(next.data.backupSchedule, "weekly");
  assert.deepEqual(next.privacy.excludedBundleIds, ["com.one", "com.two"]);
  assert.equal(next.general.locale, "fr");
  assert.equal(next.general.titleModel, "qwen-05");
  assert.equal(next.general.reduceMotion, "on");
  assert.equal(next.general.launchAtLogin, false);
  assert.equal(next.accessibility.motion, "on");
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

test("login item status stays honest when the command is missing", async () => {
  assert.equal(parseLoginItemStatus("enabled"), "enabled");
  assert.equal(parseLoginItemStatus("not_registered"), "not_registered");
  assert.equal(parseLoginItemStatus("requires_approval"), "requires_approval");
  assert.equal(parseLoginItemStatus("unavailable"), "unavailable");
  assert.equal(parseLoginItemStatus("enabled_secret"), "unavailable");
  const status = { hidden: true, dataset: {}, textContent: "" };
  const listed = await refreshLoginItemStatus(
    {
      querySelector(sel) {
        return sel === "[data-login-item-status]" ? status : null;
      },
    },
    async () => {
      throw new Error("denied");
    },
  );
  assert.equal(listed, "unavailable");
  assert.equal(status.hidden, false);
  assert.equal(status.dataset.loginItemStatus, "unavailable");
  assert.match(status.textContent, /debug build/);
  const approved = await refreshLoginItemStatus(
    {
      querySelector(sel) {
        return sel === "[data-login-item-status]" ? status : null;
      },
    },
    async (cmd) => {
      assert.equal(cmd, "login_item_status");
      return "requires_approval";
    },
  );
  assert.equal(approved, "requires_approval");
  assert.equal(status.dataset.loginItemStatus, "requires_approval");
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
    filterInstalledApps(catalog, "SAF", []).map((app) => app.bundleId),
    ["com.apple.Safari"],
  );
  const ranked = [
    { bundleId: "com.example.mybronze", name: "My Bronze" },
    { bundleId: "com.example.bronze", name: "Bronze" },
    { bundleId: "com.example.notes", name: "Notes" },
  ];
  assert.deepEqual(
    filterInstalledApps(ranked, "BRONZE", []).map((app) => app.name),
    ["Bronze", "My Bronze"],
  );
  assert.deepEqual(
    filterInstalledApps(ranked, "brn", []).map((app) => app.name),
    ["Bronze", "My Bronze"],
  );
  assert.deepEqual(filterInstalledApps(ranked, "zzzz", []), []);
  assert.deepEqual(
    filterInstalledApps(ranked, "  ", []).map((app) => app.bundleId),
    ranked.map((app) => app.bundleId),
  );
  assert.deepEqual(
    filterInstalledApps(
      [{ bundleId: "com.example.ist", name: "Istanbul" }],
      "ıstanbul",
      [],
      "tr",
    ).map((app) => app.name),
    ["Istanbul"],
  );
  assert.deepEqual(
    filterInstalledApps(
      [{ bundleId: "com.example.ist", name: "Istanbul" }],
      "istanbul",
      [],
      "tr",
    ),
    [],
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
  assert.match(html, /<option value="ru" lang="ru">🇷🇺 Русский<\/option>/);
  assert.match(html, /<option value="uk" lang="uk">🇺🇦 Українська<\/option>/);
  assert.match(html, /<option value="hr" lang="hr">🇭🇷 Hrvatski<\/option>/);
  assert.match(html, /<option value="sl" lang="sl">🇸🇮 Slovenščina<\/option>/);
  assert.match(html, /<option value="da" lang="da">🇩🇰 Dansk<\/option>/);
  assert.match(html, /<option value="sv" lang="sv">🇸🇪 Svenska<\/option>/);
  assert.match(html, /<option value="nb" lang="nb">🇳🇴 Norsk bokmål<\/option>/);
  assert.match(html, /<option value="fi" lang="fi">🇫🇮 Suomi<\/option>/);
  assert.match(html, /<option value="tr" lang="tr">🇹🇷 Türkçe<\/option>/);
  assert.match(html, /data-reset-field="general.locale"/);
  assert.match(html, /data-reset-field="general.reduceMotion"/);
  assert.match(html, /data-reset-field="general.launchAtLogin"/);
  assert.match(html, /id="launch-at-login"/);
  assert.match(html, /name="launchAtLogin"/);
  assert.match(html, /data-login-item-status/);
  assert.match(live, /login_item_status/);
  assert.match(live, /app_version_info/);
  assert.match(live, /check_for_update/);
  assert.match(live, /open_release_page/);
  assert.doesNotMatch(live, /api\.github\.com/);
  assert.doesNotMatch(live, /plugin-updater/);
  assert.doesNotMatch(live, /Sparkle/);
  assert.match(live, /#launch-at-login/);
  assert.match(html, /id="reduce-motion"/);
  assert.match(live, /emitMotionChanged/);
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
  if (sel.includes(",")) {
    return sel.split(",").some((part) => fakeMatches(node, part.trim()));
  }
  if (["label", "option", "input", "select", "textarea"].includes(sel)) {
    return node._attr.tag === sel;
  }
  if (sel.startsWith(".") && !sel.includes(" ")) {
    const cls = sel.slice(1);
    const raw = node._attr.class ?? "";
    return raw === cls || String(raw).split(/\s+/).includes(cls);
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
        attr: { "data-setting-info": "", class: "setting-info-panel" },
        text: "Sets the language of Bronze windows on this Mac",
      }),
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
  assert.match(hay, /windows on this Mac/);
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

  applySettingsSearch(root, "BCKP");
  assert.equal(data.hidden, false);
  assert.equal(dataUnit.hidden, false);
  assert.equal(general.hidden, true);
  assert.equal(shortcuts.hidden, true);
  assert.equal(empty.hidden, true);

  applySettingsSearch(root, "BACKUP");
  assert.equal(data.hidden, false);
  assert.equal(general.hidden, true);

  applySettingsSearch(root, "   ");
  assert.equal(general.hidden, false);
  assert.equal(data.hidden, false);
  assert.equal(shortcuts.hidden, false);
  assert.equal(empty.hidden, true);

  assert.equal(settingsSearchMatches("Café token", "cafe"), false);
  assert.equal(settingsSearchMatches("Café token", "CAFÉ"), true);
  assert.equal(settingsSearchMatches("Bronze", "brn"), true);
  assert.equal(settingsSearchMatches("Bronze", ""), true);
  assert.equal(settingsSearchMatches("Bronze", "zzzz"), false);
  assert.equal(settingsSearchMatches("Istanbul", "ıstanbul", "tr"), true);
  assert.equal(settingsSearchMatches("Istanbul", "istanbul", "tr"), false);
});

test("title model status shows present vs vendor command and never fetches", async () => {
  assert.deepEqual(
    [...TITLE_MODEL_IDS],
    [
      "extractive",
      "smol-135",
      "smol-360",
      "qwen-05",
      "custom",
      "ollama",
      "hosted-openai",
      "hosted-anthropic",
      "hosted-openrouter",
    ],
  );
  assert.equal(parseTitleModelId("qwen-05"), "qwen-05");
  assert.equal(parseTitleModelId("custom"), "custom");
  assert.equal(parseTitleModelId("ollama"), "ollama");
  assert.equal(parseTitleModelId("hosted-openai"), "hosted-openai");
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
      vendorCommand:
        "sh crates/bronze-title-model/scripts/vendor-gguf.sh qwen-05",
    }),
    "Vendored file missing — titles stay extractive until you run sh crates/bronze-title-model/scripts/vendor-gguf.sh qwen-05.",
  );
  assert.equal(
    formatTitleModelStatus(
      { id: "qwen-05", present: false },
      { unavailable: true },
    ),
    "Title engines could not be listed.",
  );
  const status = { textContent: "", setAttribute() {}, removeAttribute() {} };
  const spinner = { hidden: true };
  const select = { value: "qwen-05" };
  const root = {
    querySelector(sel) {
      if (sel === "[data-title-model-status]") return status;
      if (sel === "[data-title-model-spinner]") return spinner;
      if (sel === "#title-model") return select;
      return null;
    },
  };
  applyTitleModelStatus(root, {
    id: "qwen-05",
    present: false,
    vendorCommand:
      "sh crates/bronze-title-model/scripts/vendor-gguf.sh qwen-05",
  });
  assert.match(status.textContent, /vendor-gguf\.sh qwen-05/);
  assert.doesNotMatch(status.textContent, /Title:/);
  const listed = await refreshTitleModelStatus(root, async (cmd) => {
    if (cmd === "title_engine_status") {
      return { tier: "qwen-05", phase: "missing", reason: "missing_weights" };
    }
    assert.equal(cmd, "list_title_models");
    return [
      { id: "extractive", present: true, vendorCommand: "" },
      {
        id: "qwen-05",
        present: false,
        vendorCommand:
          "sh crates/bronze-title-model/scripts/vendor-gguf.sh qwen-05",
      },
    ];
  });
  assert.equal(listed?.[1]?.present, false);
  assert.match(status.textContent, /Vendored file missing/);
  assert.equal(spinner.hidden, true);
  assert.equal(
    formatTitleEngineLifecycle({ tier: "qwen-05", phase: "loading" }),
    "Loading Qwen2.5 0.5B…",
  );
  assert.equal(
    formatTitleEngineLifecycle({ tier: "qwen-05", phase: "hashing" }),
    "Checking Qwen2.5 0.5B…",
  );
  assert.equal(
    formatTitleEngineLifecycle({ tier: "qwen-05", phase: "ready" }),
    "Loaded — will title the next capture",
  );
  assert.equal(titleEngineBusy("loading"), true);
  assert.equal(titleEngineBusy("ready"), false);
  applyTitleEngineLifecycle(
    root,
    { tier: "qwen-05", phase: "loading" },
    { id: "qwen-05", present: true },
  );
  assert.match(status.textContent, /Loading Qwen2\.5 0\.5B/);
  assert.equal(spinner.hidden, false);
  applyTitleEngineLifecycle(
    root,
    { tier: "qwen-05", phase: "ready" },
    { id: "qwen-05", present: true },
  );
  assert.equal(status.textContent, "Loaded — will title the next capture");
  assert.equal(spinner.hidden, true);
  applyTitleEngineLifecycle(
    root,
    { tier: "qwen-05", phase: "missing", reason: "missing_weights" },
    {
      id: "qwen-05",
      present: false,
      vendorCommand:
        "sh crates/bronze-title-model/scripts/vendor-gguf.sh qwen-05",
    },
  );
  assert.match(status.textContent, /Vendored file missing/);
  assert.equal(spinner.hidden, true);
  assert.match(html, /id="title-model"/);
  assert.match(html, /name="titleModel"/);
  assert.match(html, /data-reset-field="general.titleModel"/);
  assert.match(html, /data-title-model-status/);
  assert.match(html, /data-title-model-spinner/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /role="status"/);
  assert.match(html, /value="extractive"/);
  assert.match(html, /value="smol-135"/);
  assert.match(html, /value="smol-360"/);
  assert.match(html, /value="qwen-05"/);
  assert.match(html, /value="custom"/);
  assert.match(html, /value="ollama"/);
  assert.match(html, /value="hosted-openai"/);
  assert.match(html, /data-import-title-gguf/);
  assert.doesNotMatch(html, /class="btn-ghost"\s+data-import-title-gguf/);
  assert.match(html, /id="title-integration"/);
  assert.match(html, /data-settings-group="titles"/);
  assert.match(html, /id="title-hosted-key"/);
  assert.match(html, /type="password"/);
  assert.match(html, /id="title-hosted-confirmed"/);
  assert.doesNotMatch(html, /download/i);
  assert.doesNotMatch(html, /huggingface/i);
  assert.match(live, /list_title_models/);
  assert.match(live, /list_ollama_title_models/);
  assert.match(live, /import_title_gguf/);
  assert.match(live, /set_hosted_title_key/);
  assert.match(live, /hosted_title_disclosure/);
  assert.match(live, /title_engine_status/);
  assert.match(live, /title-engine-status/);
  assert.match(live, /#title-model/);
  assert.match(live, /#title-integration/);
  assert.match(live, /selectedTitleModelId/);
  assert.equal(TITLE_ENGINE_STATUS_EVENT, "title-engine-status");
  assert.doesNotMatch(live, /huggingface/i);
  assert.doesNotMatch(live, /https:\/\//);
  assert.doesNotMatch(live, /apikey/i);
  assert.equal(
    formatTitleModelStatus({
      id: "custom",
      present: true,
      displayName: "tiny.gguf",
    }),
    "This imported file can title the next capture.",
  );
  assert.equal(
    formatTitleModelStatus({ id: "ollama" }),
    "Ollama will title the next capture when it is running.",
  );
  assert.equal(
    formatTitleModelStatus({ id: "hosted-openai" }, { host: "api.openai.com" }),
    "The next capture sends truncated text to api.openai.com.",
  );
  assert.match(html, /never sends text off this device/);
  assert.match(html, /Sends truncated capture text/);
});

test("title-engine-status replaces loading and a stale snapshot does not", async () => {
  const status = { textContent: "", setAttribute() {}, removeAttribute() {} };
  const spinner = { hidden: true };
  const select = { value: "smol-360" };
  const root = {
    querySelector(sel) {
      if (sel === "[data-title-model-status]") return status;
      if (sel === "[data-title-model-spinner]") return spinner;
      if (sel === "#title-model") return select;
      if (sel === "#title-integration") return { value: "none" };
      return null;
    },
  };
  let handler;
  bindTitleEngineStatus(root, (_event, next) => {
    handler = next;
    return Promise.resolve(() => {});
  });
  handler({ payload: { tier: "smol-360", phase: "loading", reason: null } });
  assert.match(status.textContent, /Loading SmolLM2 360M/);
  assert.equal(spinner.hidden, false);

  let release;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  const refresh = refreshTitleModelStatus(root, async (cmd) => {
    if (cmd === "list_title_models") {
      return [{ id: "smol-360", present: true, vendorCommand: "" }];
    }
    assert.equal(cmd, "title_engine_status");
    await gate;
    return { tier: "smol-360", phase: "loading", reason: null };
  });
  handler({ payload: { tier: "smol-360", phase: "ready", reason: null } });
  assert.equal(status.textContent, "Loaded — will title the next capture");
  assert.equal(spinner.hidden, true);
  release();
  await refresh;
  assert.equal(status.textContent, "Loaded — will title the next capture");
  assert.equal(spinner.hidden, true);

  handler({
    payload: { tier: "smol-360", phase: "failed", reason: "unreadable" },
  });
  assert.equal(
    status.textContent,
    "This file could not be read, so titles stay extractive.",
  );
  assert.equal(spinner.hidden, true);

  handler({
    payload: {
      tier: "smol-360",
      phase: "missing",
      reason: "missing_weights",
    },
  });
  assert.match(status.textContent, /Vendored file missing/);
  assert.equal(spinner.hidden, true);

  let releaseReady;
  const readyGate = new Promise((resolve) => {
    releaseReady = resolve;
  });
  const readyRefresh = refreshTitleModelStatus(root, async (cmd) => {
    if (cmd === "list_title_models") {
      return [{ id: "smol-360", present: true, vendorCommand: "" }];
    }
    await readyGate;
    return { tier: "smol-360", phase: "ready", reason: null };
  });
  handler({ payload: { tier: "smol-360", phase: "loading", reason: null } });
  assert.match(status.textContent, /Loading SmolLM2 360M/);
  releaseReady();
  await readyRefresh;
  assert.equal(status.textContent, "Loaded — will title the next capture");
  assert.equal(spinner.hidden, true);
});

test("imported GGUF option shows RAM from file size and keeps two CPU threads", () => {
  assert.equal(formatTitleFileSize(270 * 1024 * 1024), "270 MB");
  assert.equal(
    formatCustomTitleOption(0),
    "Imported GGUF — RAM follows the file, 2 CPU threads",
  );
  assert.equal(
    formatCustomTitleOption(270 * 1024 * 1024),
    "Imported GGUF — about 270 MB of RAM, 2 CPU threads",
  );
  const customOption = { textContent: "" };
  const file = { textContent: "" };
  const root = {
    querySelector(sel) {
      if (sel === '#title-model option[value="custom"]') return customOption;
      if (sel === "[data-title-custom-file]") return file;
      return null;
    },
  };
  syncTitleEnginePanels(root, {
    general: {
      titleCustomName: "tiny.gguf",
      titleCustomBytes: 105 * 1024 * 1024,
    },
  });
  assert.equal(
    customOption.textContent,
    "Imported GGUF — about 105 MB of RAM, 2 CPU threads",
  );
  assert.match(file.textContent, /tiny\.gguf/);
  assert.match(file.textContent, /105 MB/);
});

test("title engine shows one source: local picker or integration form", () => {
  const titles = { dataset: {} };
  const hosted = { hidden: true };
  const ollama = { hidden: true };
  const custom = { hidden: true };
  const titleIntegration = { value: "hosted-openai" };
  const titleModel = { value: "smol-360" };
  const root = {
    querySelector(sel) {
      if (sel === '[data-settings-group="titles"]') return titles;
      if (sel === "#title-integration") return titleIntegration;
      if (sel === "#title-model") return titleModel;
      if (sel === '[data-title-engine-panel="hosted"]') return hosted;
      if (sel === '[data-title-engine-panel="ollama"]') return ollama;
      if (sel === '[data-title-engine-panel="custom"]') return custom;
      return null;
    },
  };
  syncTitleEnginePanels(root, { general: { titleModel: "hosted-openai" } });
  assert.equal(titles.dataset.titleEngineSource, "integration");
  assert.equal(hosted.hidden, false);
  assert.equal(ollama.hidden, true);
  titleIntegration.value = "none";
  syncTitleEnginePanels(root, { general: { titleModel: "smol-360" } });
  assert.equal(titles.dataset.titleEngineSource, "local");
  assert.equal(hosted.hidden, true);
  titleIntegration.value = "ollama";
  syncTitleEnginePanels(root, { general: { titleModel: "ollama" } });
  assert.equal(titles.dataset.titleEngineSource, "integration");
  assert.equal(ollama.hidden, false);
  assert.equal(hosted.hidden, true);
});

test("setting info opens on hover and focus, not click, and dismisses on Escape", () => {
  const listeners = [];
  const makeInfo = (name) => {
    const summary = {
      focus() {},
      addEventListener(type, handler) {
        listeners.push([name, `summary:${type}`, handler]);
      },
    };
    return {
      open: false,
      ownerDocument: { activeElement: null },
      querySelector: () => summary,
      contains() {
        return false;
      },
      matches() {
        return false;
      },
      addEventListener(type, handler) {
        listeners.push([name, type, handler]);
      },
    };
  };
  const first = makeInfo("first");
  const second = makeInfo("second");
  const root = {
    querySelectorAll(sel) {
      return sel === "details.setting-info" ? [first, second] : [];
    },
    querySelector() {
      return null;
    },
    addEventListener(type, handler) {
      listeners.push(["root", type, handler]);
    },
  };
  bindSettingInfo(root);
  listeners.find((row) => row[0] === "first" && row[1] === "pointerenter")[2]();
  assert.equal(first.open, true);
  assert.equal(second.open, false);
  listeners.find(
    (row) => row[0] === "second" && row[1] === "pointerenter",
  )[2]();
  assert.equal(first.open, false);
  assert.equal(second.open, true);
  let prevented = false;
  listeners.find((row) => row[0] === "second" && row[1] === "summary:click")[2](
    {
      preventDefault() {
        prevented = true;
      },
    },
  );
  assert.equal(prevented, true);
  assert.equal(second.open, true);
  listeners.find(
    (row) => row[0] === "second" && row[1] === "pointerleave",
  )[2]();
  assert.equal(second.open, false);
  listeners.find((row) => row[0] === "first" && row[1] === "focusin")[2]();
  assert.equal(first.open, true);
  listeners.find((row) => row[0] === "root" && row[1] === "keydown")[2]({
    key: "Escape",
    preventDefault() {},
  });
  assert.equal(first.open, false);
  listeners.find((row) => row[0] === "first" && row[1] === "pointerenter")[2]();
  assert.equal(first.open, false);
  listeners.find((row) => row[0] === "first" && row[1] === "pointerleave")[2]();
  listeners.find((row) => row[0] === "first" && row[1] === "pointerenter")[2]();
  assert.equal(first.open, true);
  listeners.find((row) => row[0] === "root" && row[1] === "pointerdown")[2]({
    target: { closest: () => null },
  });
  assert.equal(first.open, false);
});

test("app version loads locally and check is user-initiated", async () => {
  assert.equal(parseInstallSource("homebrew"), "homebrew");
  assert.equal(parseInstallSource("cellar"), "unknown");
  assert.equal(parseUpdateAction("brew-upgrade"), "brew-upgrade");
  assert.equal(parseUpdateAction("sparkle"), "none");
  const number = { textContent: "" };
  const source = { hidden: true, dataset: {}, textContent: "" };
  const status = { hidden: true, textContent: "" };
  const dialog = { showModal() {} };
  const opened = [];
  dialog.showModal = () => opened.push("dialog");
  const actionBtn = {
    hidden: true,
    dataset: {},
    textContent: "",
  };
  const available = { textContent: "" };
  const notes = {
    ownerDocument: {
      createElement(tag) {
        const node = {
          tag,
          className: "",
          textContent: "",
          children: [],
          append(child) {
            this.children.push(child);
          },
        };
        return node;
      },
    },
    children: [],
    replaceChildren(...nodes) {
      this.children = nodes;
    },
  };
  const debugLine = { hidden: true };
  const commandEl = { hidden: true, textContent: "" };
  const root = {
    querySelector(sel) {
      if (sel === "[data-app-version-number]") return number;
      if (sel === "[data-app-version-source]") return source;
      if (sel === "[data-app-version-status]") return status;
      if (sel === "#update-sheet") return dialog;
      if (sel === "[data-update-action]") return actionBtn;
      if (sel === "[data-update-available]") return available;
      if (sel === "[data-update-notes]") return notes;
      if (sel === "[data-update-debug]") return debugLine;
      if (sel === "[data-update-command]") return commandEl;
      return null;
    },
  };
  const commands = [];
  const info = await refreshAppVersion(root, async (cmd) => {
    commands.push(cmd);
    assert.equal(cmd, "app_version_info");
    return { version: "0.1.0", installSource: "direct" };
  });
  assert.equal(info.version, "0.1.0");
  assert.equal(number.textContent, "0.1.0");
  assert.equal(source.dataset.appVersionSource, "direct");
  assert.deepEqual(commands, ["app_version_info"]);
  applyAppVersionInfo(root, { version: "", installSource: "debug" });
  assert.match(number.textContent, /unavailable|0\.1\.0|Version/);
  const current = await checkForAppUpdate(root, async (cmd) => {
    commands.push(cmd);
    assert.equal(cmd, "check_for_update");
    return {
      available: false,
      latestVersion: "0.1.0",
      notes: [],
      action: "none",
    };
  });
  assert.equal(current.available, false);
  assert.equal(opened.length, 0);
  const brew = applyUpdateCheck(root, {
    available: true,
    latestVersion: "0.2.0",
    notes: [
      "Check for updates is easier to see. https://github.com/NxT-Solutions/Bronze/pull/44",
      "",
    ],
    action: "brew-upgrade",
    actionCommand: "brew upgrade --cask bronze",
    releaseUrl: "https://github.com/NxT-Solutions/Bronze/releases/tag/v0.2.0",
  });
  assert.equal(brew.available, true);
  assert.deepEqual(opened, ["dialog"]);
  assert.equal(actionBtn.hidden, false);
  assert.equal(actionBtn.dataset.updateAction, "brew-upgrade");
  assert.equal(actionBtn.dataset.actionCommand, "brew upgrade --cask bronze");
  assert.equal(available.textContent, "Version 0.2.0 is available.");
  assert.equal(notes.children[0].tag, "ul");
  assert.equal(
    notes.children[0].children[0].textContent,
    "Check for updates is easier to see.",
  );
  assert.ok(
    notes.children[0].children.every(
      (item) =>
        !item.textContent.includes("http") && item.textContent.includes(" "),
    ),
  );
  const debug = applyUpdateCheck(root, {
    available: true,
    latestVersion: "9.0.0",
    notes: ["Debug notes"],
    action: "debug",
  });
  assert.equal(debug.action, "debug");
  assert.equal(actionBtn.hidden, true);
  assert.equal(debugLine.hidden, false);
  renderUpdateNotes(notes, []);
  assert.equal(notes.children[0].tag, "p");
  assert.equal(
    displayUpdateNote("Check for updates is easier to see.", (key) =>
      key === "settings.field.version.note.checkForUpdates"
        ? "Nach Updates suchen ist leichter zu sehen."
        : "",
    ),
    "Nach Updates suchen ist leichter zu sehen.",
  );
  assert.equal(
    displayUpdateNote(
      "fix(storage): retune sqlite wal checkpoint pragma by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/99",
    ),
    "",
  );
  assert.equal(
    displayUpdateNote("Version 0.1.2 is available."),
    "Version 0.1.2 is available.",
  );
});
