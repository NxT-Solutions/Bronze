import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyCatalogStrings,
  applyHandTestLocale,
  bindHandTestLocale,
  catalogHasRealSpaces,
  EN_HAND_TEST_CATALOG,
  expandHandTestString,
  HAND_TEST_DEFAULT_LOCALE,
  looksSmashedLocale,
  resolveUiLocale,
} from "./apply-locale.mjs";

const rootDir = dirname(fileURLToPath(import.meta.url));
const settingsHtml = readFileSync(join(rootDir, "settings.html"), "utf8");
const enFile = JSON.parse(
  readFileSync(
    join(rootDir, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

function nodesFromSettingsMarkup() {
  const nodes = [];
  const re = /data-i18n="([^"]+)"[^>]*>([\s\S]*?)</g;
  let match = re.exec(settingsHtml);
  while (match) {
    const key = match[1];
    const text = match[2].replace(/\s+/g, " ").trim();
    nodes.push({
      getAttribute: () => key,
      textContent: text,
    });
    match = re.exec(settingsHtml);
  }
  return nodes;
}

test("hand-test locale stays en unless a pseudo locale is explicit", () => {
  assert.equal(HAND_TEST_DEFAULT_LOCALE, "en");
  assert.equal(resolveUiLocale("system"), "en");
  assert.equal(resolveUiLocale("en"), "en");
  assert.equal(resolveUiLocale("en-XA"), "en-XA");
  assert.equal(resolveUiLocale(undefined), "en");
  assert.equal(
    catalogHasRealSpaces("Needed for the global capture chord"),
    true,
  );
  assert.equal(
    catalogHasRealSpaces("Needededforthegloballcaptureechordd"),
    false,
  );
  assert.equal(looksSmashedLocale("Listteningg", "Listening"), true);
  assert.equal(looksSmashedLocale("Listening", "Listening"), false);
  assert.equal(
    looksSmashedLocale("Needededforthegloballcaptureechordd", "Needed for"),
    true,
  );
});

test("en catalog titles paint shortcut labels and refuse mid-word smash", () => {
  const title = {
    getAttribute: () => "settings.shortcuts.action.app.togglePanel",
    textContent: "Toggle panel",
  };
  const smashed = {
    getAttribute: () => "settings.shortcuts.live",
    textContent: "Listening",
  };
  const root = {
    documentElement: { lang: "", dir: "" },
    querySelectorAll() {
      return [title, smashed];
    },
  };
  assert.equal(
    applyHandTestLocale(root, "system", {
      "settings.shortcuts.action.app.togglePanel": "Toggle panel",
      "settings.shortcuts.live": "Listening",
    }),
    "en",
  );
  assert.equal(root.documentElement.lang, "en");
  assert.equal(title.textContent, "Toggle panel");
  applyCatalogStrings(root, {
    "settings.shortcuts.action.app.togglePanel": "Toggle panel",
    "settings.shortcuts.live": "Listteningg",
  });
  assert.equal(title.textContent, "Toggle panel");
  assert.equal(smashed.textContent, "Listening");
  applyCatalogStrings(root, {
    "settings.shortcuts.live": "Needededforthegloballcaptureechordd",
  });
  assert.equal(smashed.textContent, "Listening");
});

test("en hand-test does not run a last-letter doubling transform", () => {
  assert.equal(expandHandTestString("en", "Shortcuts"), "Shortcuts");
  assert.equal(expandHandTestString("system", "Skip test"), "Skip test");
  assert.notEqual(expandHandTestString("en", "Shortcuts"), "Shortcutts");
  assert.notEqual(expandHandTestString("en", "Skip test"), "Skipptestt");
  assert.equal(EN_HAND_TEST_CATALOG["settings.shortcuts.title"], "Shortcuts");
  assert.equal(
    EN_HAND_TEST_CATALOG["settings.shortcuts.skipTest"],
    "Skip test",
  );
  assert.equal(
    EN_HAND_TEST_CATALOG["settings.shortcuts.title"],
    enFile["settings.shortcuts.title"],
  );
});

test("bindHandTestLocale paints en catalog over smashed shortcut headings", async () => {
  const heading = {
    getAttribute: () => "settings.shortcuts.title",
    textContent: "Shortcutts",
  };
  const skip = {
    getAttribute: () => "settings.shortcuts.skipTest",
    textContent: "Skipptestt",
  };
  const live = {
    getAttribute: () => "settings.shortcuts.live",
    textContent: "Listteningg",
  };
  const root = {
    documentElement: { lang: "", dir: "" },
    querySelectorAll() {
      return [heading, skip, live];
    },
  };
  assert.equal(await bindHandTestLocale(root, async () => "en"), "en");
  assert.equal(heading.textContent, "Shortcuts");
  assert.equal(skip.textContent, "Skip test");
  assert.equal(live.textContent, "Listening");
});

test("rendered settings markup plus en catalog never contains Shortcutts", () => {
  const nodes = nodesFromSettingsMarkup();
  const root = {
    documentElement: { lang: "", dir: "" },
    querySelectorAll() {
      return nodes;
    },
  };
  applyHandTestLocale(root, "en");
  const innerText = nodes.map((node) => node.textContent).join("\n");
  assert.match(innerText, /Shortcuts/);
  assert.match(innerText, /Skip test/);
  assert.match(innerText, /Listening/);
  assert.doesNotMatch(innerText, /Shortcutts|Skipptestt|Listteningg|Captuure/);
  assert.equal(
    nodes.filter((node) => node.textContent === "Capture selection").length,
    1,
  );
});
