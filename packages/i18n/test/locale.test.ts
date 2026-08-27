import { describe, expect, it } from "vitest";
import { createI18n } from "../src/create-i18n";
import {
  canonicalizeBcp47,
  computeFallbackChain,
  isSupported,
} from "../src/locale";

describe("locale fallback", () => {
  it("computes script-preserving chain for zh-Hant-HK", () => {
    expect(computeFallbackChain("zh-Hant-HK")).toEqual([
      "zh-Hant-HK",
      "zh-Hant",
      "zh",
      "en",
    ]);
  });
  it("handles base+region", () => {
    expect(computeFallbackChain("pt-BR")).toEqual(["pt-BR", "pt", "en"]);
  });
  it("canonicalizes", () => {
    expect(canonicalizeBcp47("ZH_hant_hk")).toMatch(/zh-Hant-HK/);
  });
  it("isSupported for seeded", () => {
    expect(isSupported("en")).toBe(true);
    expect(isSupported("en-XA")).toBe(true);
    expect(isSupported("ar-XB")).toBe(true);
    expect(isSupported("fr")).toBe(false);
  });
});

describe("i18n ICU plural", () => {
  it("formats plural via ICU", async () => {
    const i18n = await createI18n({ lng: "en" });
    const t = i18n.getFixedT("en", "app");
    expect(t("queue.count", { count: 0 })).toContain("0 items");
    expect(t("queue.count", { count: 1 })).toContain("1 item");
    expect(t("queue.count", { count: 2 })).toContain("2 items");
    // pseudo should also work if keys match
    const i18nXA = await createI18n({ lng: "en-XA" });
    const txa = i18nXA.getFixedT("en-XA", "app");
    expect(txa("queue.count", { count: 1 })).toMatch(/【.*1 item.*】/);
    // script-preserving fallback must reach en for unknown script tag
    const i18nZH = await createI18n({ lng: "zh-Hant-HK" });
    const tzh = i18nZH.getFixedT("zh-Hant-HK", "app");
    expect(tzh("app.name")).toBe("Bronze");
  });

  it("uses en when pt-BR catalog is absent", async () => {
    const i18n = await createI18n({ lng: "pt-BR" });
    const t = i18n.getFixedT("pt-BR", "app");
    expect(t("app.name")).toBe("Bronze");
    expect(t("capture.source", { appName: "Mail" })).toBe("From Mail");
  });

  it("isolates user-supplied interpolation values", async () => {
    const i18n = await createI18n({ lng: "en" });
    const t = i18n.getFixedT("en", "app");
    const userText = "<<user-supplied>>";
    const out = t("capture.source", { appName: userText });
    expect(out).toBe("From <<user-supplied>>");
    expect(out).not.toBe(userText);
    expect(t(userText)).not.toBe(out);
  });
});
