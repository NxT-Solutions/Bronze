import { describe, expect, it } from "vitest";
import {
  ADVERTISED_LOCALES,
  cardLangDir,
  chromeDir,
  INFOPLIST_GLOSSARY,
  NATIVE_GLOSSARY_KEYS,
} from "@/lib/catalog";

describe("catalog glossary (I18N-001)", () => {
  it("names native and InfoPlist keys and card isolation", () => {
    expect(ADVERTISED_LOCALES).toEqual([
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
    for (const tag of ["ru", "uk", "hr", "sl", "da", "sv", "nb", "fi", "tr"]) {
      expect(chromeDir(tag)).toBe("ltr");
    }
    expect(NATIVE_GLOSSARY_KEYS).toContain("menu.status.capture");
    expect(INFOPLIST_GLOSSARY.map(([, key]) => key)).toContain("app.name");
    expect(chromeDir("ar-XB")).toBe("rtl");
    expect(chromeDir("en-XA")).toBe("ltr");
    expect(cardLangDir("ja")).toEqual({ lang: "ja", dir: "auto" });
    expect(cardLangDir(undefined)).toEqual({ lang: "und", dir: "auto" });
  });
});
