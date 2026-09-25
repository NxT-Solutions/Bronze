import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  ADVERTISED_LOCALES,
  cardLangDir,
  chromeDir,
  INFOPLIST_GLOSSARY,
  NATIVE_GLOSSARY_KEYS,
  PANEL_CHROME_KEYS,
} from "../src/catalog";
import { KNOWN_MESSAGE_IDS } from "../src/generated-message-ids";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "locales");

function load(locale: string): Record<string, string> {
  return JSON.parse(readFileSync(join(root, locale, "app.json"), "utf8"));
}

describe("native and WebView catalog parity (I18N-001/002/003/004, G-06)", () => {
  it("covers menu and InfoPlist keys in en, en-XA, and ar-XB", () => {
    for (const locale of ADVERTISED_LOCALES) {
      const catalog = load(locale);
      for (const key of NATIVE_GLOSSARY_KEYS) {
        expect(catalog[key], `${locale} ${key}`).toBeTruthy();
        expect(KNOWN_MESSAGE_IDS).toContain(key);
      }
      for (const [plist, key] of INFOPLIST_GLOSSARY) {
        expect(catalog[key], `${locale} ${plist}`).toBeTruthy();
      }
      for (const key of PANEL_CHROME_KEYS) {
        expect(catalog[key], `${locale} chrome ${key}`).toBeTruthy();
      }
    }
  });

  it("translates spaced source strings for the added UI locales", () => {
    const english = load("en");
    const mayMatchEnglish = new Set([
      "app.name",
      "panel.quick.title",
      "settings.field.titleModel.engine.smol135",
      "settings.field.titleModel.engine.smol360",
      "settings.field.titleModel.engine.qwen05",
      "settings.field.titleModel.engine.ollama",
      "settings.shortcuts.key.arrowUp",
      "settings.shortcuts.key.arrowDown",
      "settings.shortcuts.modifier.fn",
    ]);
    for (const locale of [
      "ru",
      "uk",
      "hr",
      "sl",
      "da",
      "sv",
      "nb",
      "fi",
      "tr",
    ]) {
      const catalog = load(locale);
      expect(Object.keys(catalog).sort()).toEqual(Object.keys(english).sort());
      for (const [key, value] of Object.entries(english)) {
        if (mayMatchEnglish.has(key)) continue;
        if (/[A-Za-z]/.test(value) && /\s/.test(value)) {
          expect(catalog[key], `${locale} ${key}`).not.toBe(value);
        }
      }
    }
  });

  it("maps chrome dir for RTL smoke and isolates card lang/dir", () => {
    expect(chromeDir("ar-XB")).toBe("rtl");
    expect(chromeDir("en-XA")).toBe("ltr");
    for (const tag of ["ru", "uk", "hr", "sl", "da", "sv", "nb", "fi", "tr"]) {
      expect(chromeDir(tag)).toBe("ltr");
    }
    expect(cardLangDir("ja")).toEqual({ lang: "ja", dir: "auto" });
    expect(cardLangDir(undefined)).toEqual({ lang: "und", dir: "auto" });
  });
});
