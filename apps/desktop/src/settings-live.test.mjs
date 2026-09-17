import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applySettingsForm,
  applySettingsSearch,
  patchSettingsFromForm,
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
