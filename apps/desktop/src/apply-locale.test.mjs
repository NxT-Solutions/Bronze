import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyCatalogRichText,
  applyHandTestLocale,
  catalogHasRealSpaces,
  emitUiLocaleChanged,
  formatQueueCount,
  HAND_TEST_DEFAULT_LOCALE,
  htmlLangFor,
  LOCALE_APPLIED_EVENT,
  listenUiLocaleChanged,
  resolveUiLocale,
  SHIPPED_UI_LOCALES,
  UI_LOCALE_EVENT,
} from "./apply-locale.mjs";

const applyLocaleSource = readFileSync(
  join(dirname(fileURLToPath(import.meta.url)), "apply-locale.mjs"),
  "utf8",
);

test("hand-test locale stays en unless a shipped or pseudo locale is explicit", () => {
  assert.equal(HAND_TEST_DEFAULT_LOCALE, "en");
  assert.equal(resolveUiLocale("system"), "en");
  assert.equal(resolveUiLocale("en"), "en");
  assert.equal(resolveUiLocale("nl"), "nl");
  assert.equal(resolveUiLocale("nl-BE"), "nl");
  assert.equal(resolveUiLocale("fr"), "fr");
  assert.equal(resolveUiLocale("de"), "de");
  assert.equal(resolveUiLocale("es"), "es");
  assert.equal(resolveUiLocale("it"), "it");
  assert.equal(resolveUiLocale("ru"), "ru");
  assert.equal(resolveUiLocale("uk"), "uk");
  assert.equal(resolveUiLocale("hr"), "hr");
  assert.equal(resolveUiLocale("sl"), "sl");
  assert.equal(resolveUiLocale("da"), "da");
  assert.equal(resolveUiLocale("sv"), "sv");
  assert.equal(resolveUiLocale("nb"), "nb");
  assert.equal(resolveUiLocale("fi"), "fi");
  assert.equal(resolveUiLocale("tr"), "tr");
  assert.equal(resolveUiLocale("sv-SE"), "sv");
  assert.equal(resolveUiLocale("nn"), "en");
  assert.equal(resolveUiLocale("en-XA"), "en-XA");
  assert.equal(resolveUiLocale(undefined), "en");
  assert.deepEqual(SHIPPED_UI_LOCALES, [
    "en",
    "nl",
    "fr",
    "de",
    "es",
    "it",
    "ru",
    "uk",
    "hr",
    "sl",
    "da",
    "sv",
    "nb",
    "fi",
    "tr",
    "en-XA",
    "ar-XB",
  ]);
  assert.equal(htmlLangFor("nl"), "nl");
  assert.equal(htmlLangFor("ru"), "ru");
  assert.equal(htmlLangFor("tr"), "tr");
  assert.equal(htmlLangFor("ar-XB"), "ar");
  assert.equal(
    catalogHasRealSpaces("Needed for the global capture chord"),
    true,
  );
  assert.equal(
    catalogHasRealSpaces("Needededforthegloballcaptureechordd"),
    false,
  );
});

test("setting info catalog bolds markers and treats markup as text", () => {
  const kids = [];
  const el = {
    ownerDocument: {
      createElement(tag) {
        return { tagName: tag, textContent: "" };
      },
      createTextNode(text) {
        return { nodeType: 3, textContent: text };
      },
      createDocumentFragment() {
        const nodes = [];
        return {
          append(...part) {
            nodes.push(...part);
          },
          nodes,
        };
      },
    },
    replaceChildren(frag) {
      kids.splice(0, kids.length, ...(frag.nodes ?? []));
    },
  };
  applyCatalogRichText(el, "**Record** a row to replace that shortcut.");
  assert.equal(kids[0].tagName, "strong");
  assert.equal(kids[0].textContent, "Record");
  assert.equal(kids[1].textContent, " a row to replace that shortcut.");
  applyCatalogRichText(el, "**Stays on this Mac** and works offline.");
  assert.equal(kids[0].tagName, "strong");
  assert.equal(kids[0].textContent, "Stays on this Mac");
  assert.equal(kids[1].textContent, " and works offline.");
  applyCatalogRichText(el, "<b>unsafe</b>");
  assert.equal(kids.length, 1);
  assert.equal(kids[0].nodeType, 3);
  assert.equal(kids[0].textContent, "<b>unsafe</b>");
  assert.match(applyLocaleSource, /includes\("field-help"\)/);
});

test("applyHandTestLocale sets html lang and catalog chrome", () => {
  const title = { textContent: "Settings" };
  const heading = {
    textContent: "Settings",
    getAttribute: () => "settings.title",
  };
  const nodes = [heading];
  const root = {
    documentElement: { lang: "en", dir: "ltr" },
    getElementById(id) {
      return id === "settings" ? {} : null;
    },
    querySelector(sel) {
      return sel === "title" ? title : null;
    },
    querySelectorAll(sel) {
      if (sel === "[data-i18n]") {
        return nodes;
      }
      return [];
    },
  };
  heading.closest = () => null;
  const resolved = applyHandTestLocale(root, "nl", {
    locale: "nl",
    htmlLang: "nl",
    dir: "ltr",
    catalogAvailable: true,
    messages: { "settings.title": "Instellingen" },
  });
  assert.equal(resolved, "nl");
  assert.equal(root.documentElement.lang, "nl");
  assert.equal(heading.textContent, "Instellingen");
  assert.equal(title.textContent, "Instellingen");
  assert.match(applyLocaleSource, /setAttribute\("aria-placeholder", value\)/);
  assert.match(applyLocaleSource, /setAttribute\("title", value\)/);
  assert.match(applyLocaleSource, /setAttribute\("aria-label", value\)/);
});

test("formatQueueCount uses ICU branches without concatenation", () => {
  const nl = "{count, plural, =0 {0 items} one {1 item} other {# items}}";
  const fr =
    "{count, plural, =0 {0 élément} one {1 élément} other {# éléments}}";
  assert.equal(formatQueueCount(0, nl), "0 items");
  assert.equal(formatQueueCount(2, nl), "2 items");
  assert.equal(formatQueueCount(0, fr), "0 élément");
  assert.equal(formatQueueCount(1, fr), "1 élément");
  assert.equal(formatQueueCount(3, fr), "3 éléments");
  assert.equal(formatQueueCount(2, ""), "2 items");
  const ru =
    "{count, plural, =0 {0 элементов} one {# элемент} few {# элемента} many {# элементов} other {# элемента}}";
  assert.equal(formatQueueCount(0, ru, "ru"), "0 элементов");
  assert.equal(formatQueueCount(1, ru, "ru"), "1 элемент");
  assert.equal(formatQueueCount(2, ru, "ru"), "2 элемента");
  assert.equal(formatQueueCount(5, ru, "ru"), "5 элементов");
  assert.equal(formatQueueCount(21, ru, "ru"), "21 элемент");
});

test("locale change is broadcast so every open window can reapply", async () => {
  assert.equal(UI_LOCALE_EVENT, "ui-locale-changed");
  assert.equal(LOCALE_APPLIED_EVENT, "bronze:locale-changed");
  if (typeof BroadcastChannel !== "function") {
    return;
  }
  const seen = [];
  const stop = listenUiLocaleChanged((payload) => {
    seen.push(payload);
  });
  await emitUiLocaleChanged({ locale: "nl" });
  await new Promise((resolve) => setTimeout(resolve, 20));
  assert.equal(
    seen.some((payload) => payload?.locale === "nl"),
    true,
  );
  stop();
});
