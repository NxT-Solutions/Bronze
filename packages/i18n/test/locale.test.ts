import { getI18n } from "react-i18next";
import { describe, expect, it } from "vitest";
import { createI18n } from "../src/create-i18n";
import {
  getCollator,
  getDateTimeFormat,
  getDisplayNames,
  getListFormat,
  getNumberFormat,
  getPluralRules,
  getRelativeTimeFormat,
  getSegmenter,
} from "../src/formatters";
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
    expect(isSupported("nl")).toBe(true);
    expect(isSupported("fr")).toBe(true);
    expect(isSupported("de")).toBe(true);
    expect(isSupported("es")).toBe(true);
    expect(isSupported("it")).toBe(true);
    expect(isSupported("pt")).toBe(false);
  });
});

describe("i18n ICU plural", () => {
  it("formats plural via ICU", async () => {
    const { t } = await createI18n({ lng: "en" });
    expect(t("queue.count", { count: 0 })).toContain("0 items");
    expect(t("queue.count", { count: 1 })).toContain("1 item");
    expect(t("queue.count", { count: 2 })).toContain("2 items");
    // pseudo should also work if keys match
    const { t: txa } = await createI18n({ lng: "en-XA" });
    expect(txa("queue.count", { count: 1 })).toMatch(/【.*1 item.*】/);
    // script-preserving fallback must reach en for unknown script tag
    const { t: tzh } = await createI18n({ lng: "zh-Hant-HK" });
    expect(tzh("app.name")).toBe("Bronze");
    const { t: tfr } = await createI18n({ lng: "fr" });
    expect(tfr("settings.field.locale")).toBe("Langue");
    expect(tfr("settings.field.titleModel")).toBe("Moteur de titre");
    expect(
      tfr("settings.field.titleModel.missing", {
        command: "sh bronze-title-model/scripts/vendor-gguf.sh qwen-05",
      }),
    ).toBe(
      "Fichier vendor manquant; les titres restent extractifs, exécutez sh bronze-title-model/scripts/vendor-gguf.sh qwen-05",
    );
    expect(tfr("capture.source", { appName: "Mail" })).toBe("De Mail");
  });

  it("uses en when pt-BR catalog is absent", async () => {
    const { t } = await createI18n({ lng: "pt-BR" });
    expect(t("app.name")).toBe("Bronze");
    expect(t("capture.source", { appName: "Mail" })).toBe("From Mail");
  });

  it("isolates user-supplied interpolation values", async () => {
    const { t } = await createI18n({ lng: "en" });
    const userText = "<<user-supplied>>";
    const out = t("capture.source", { appName: userText });
    expect(out).toBe("From <<user-supplied>>");
    expect(out).not.toBe(userText);
    expect(
      (t as (k: string, o?: Record<string, unknown>) => string)(userText),
    ).not.toBe(out);
  });
});

describe("formatters (all 8, cached)", () => {
  it("caches by (locale, opts) identity for new formatters", () => {
    const a = getRelativeTimeFormat("en", { numeric: "auto" });
    const b = getRelativeTimeFormat("en", { numeric: "auto" });
    expect(a).toBe(b);
    expect(a).toBeInstanceOf(Intl.RelativeTimeFormat);
    expect(a.format(-1, "day")).toMatch(/1 day ago|yesterday/);
    expect(getRelativeTimeFormat("en")).toBe(getRelativeTimeFormat("en"));

    const c = getListFormat("en");
    const d = getListFormat("en");
    expect(c).toBe(d);
    expect(c).toBeInstanceOf(Intl.ListFormat);
    expect(c.format(["a", "b"])).toContain("a");
    expect(getListFormat("en", { style: "long", type: "conjunction" })).toBe(
      getListFormat("en", { type: "conjunction", style: "long" }),
    );

    const e = getCollator("en");
    const f = getCollator("en");
    expect(e).toBe(f);
    expect(e).toBeInstanceOf(Intl.Collator);
    expect(e.compare("a", "b")).toBeLessThan(0);

    const g = getDisplayNames("en", { type: "language" });
    const h = getDisplayNames("en", { type: "language" });
    expect(g).toBe(h);
    expect(g).toBeInstanceOf(Intl.DisplayNames);
    expect(g.of("en")).toBeTypeOf("string");
    expect(getDisplayNames("en")).toBe(
      getDisplayNames("en", { type: "language" }),
    );
    expect(getDisplayNames("en", {} as Intl.DisplayNamesOptions)).toBe(
      getDisplayNames("en"),
    );

    const i = getSegmenter("en");
    const j = getSegmenter("en");
    expect(i).toBe(j);
    expect(i).toBeInstanceOf(Intl.Segmenter);
    expect([...i.segment("ab")].length).toBeGreaterThan(0);
  });

  it("exposes the five added getters and existing still work", () => {
    expect(getNumberFormat("en")).toBe(getNumberFormat("en"));
    expect(getNumberFormat("en")).toBeInstanceOf(Intl.NumberFormat);
    expect(getDateTimeFormat("en")).toBe(getDateTimeFormat("en"));
    expect(getDateTimeFormat("en")).toBeInstanceOf(Intl.DateTimeFormat);
    expect(getPluralRules("en")).toBe(getPluralRules("en"));
    expect(getPluralRules("en")).toBeInstanceOf(Intl.PluralRules);
  });
});

describe("adapter surface (matrix coverage)", () => {
  it("TYPED_T_BAD_KEY (compile-time enforced by MessageId in TFunction sig)", async () => {
    const { t } = await createI18n({ lng: "en" });
    expect(t("app.name")).toBe("Bronze");
    // @ts-expect-error -- key is not a MessageId
    expect(t("no.such.key")).toBe("no.such.key");
  });
  it("REACT_SIDE_EFFECT: createI18n performs the init side-effects that enable react-i18next hooks", async () => {
    const adapter = await createI18n({ lng: "en" });
    expect(Object.keys(adapter)).toEqual(["t"]);
    expect(getI18n().t("app.name")).toBe("Bronze");
  });
});
