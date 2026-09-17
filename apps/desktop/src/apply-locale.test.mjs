import assert from "node:assert/strict";
import { test } from "node:test";
import {
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
  assert.equal(resolveUiLocale("en-XA"), "en-XA");
  assert.equal(resolveUiLocale(undefined), "en");
  assert.deepEqual(SHIPPED_UI_LOCALES, [
    "en",
    "nl",
    "fr",
    "de",
    "es",
    "it",
    "en-XA",
    "ar-XB",
  ]);
  assert.equal(htmlLangFor("nl"), "nl");
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
