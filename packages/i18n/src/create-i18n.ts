import type { i18n as I18n } from "i18next";
import i18next from "i18next";
import ICU from "i18next-icu";
import { initReactI18next } from "react-i18next";
import arXBApp from "../locales/ar-XB/app.json" with { type: "json" };
import daApp from "../locales/da/app.json" with { type: "json" };
import deApp from "../locales/de/app.json" with { type: "json" };
import enApp from "../locales/en/app.json" with { type: "json" };
import enXAApp from "../locales/en-XA/app.json" with { type: "json" };
import esApp from "../locales/es/app.json" with { type: "json" };
import fiApp from "../locales/fi/app.json" with { type: "json" };
import frApp from "../locales/fr/app.json" with { type: "json" };
import hrApp from "../locales/hr/app.json" with { type: "json" };
import itApp from "../locales/it/app.json" with { type: "json" };
import nbApp from "../locales/nb/app.json" with { type: "json" };
import nlApp from "../locales/nl/app.json" with { type: "json" };
import ruApp from "../locales/ru/app.json" with { type: "json" };
import slApp from "../locales/sl/app.json" with { type: "json" };
import svApp from "../locales/sv/app.json" with { type: "json" };
import trApp from "../locales/tr/app.json" with { type: "json" };
import ukApp from "../locales/uk/app.json" with { type: "json" };
import type { MessageId } from "./generated-message-ids";
import { computeFallbackChain } from "./locale";

const resources = {
  en: { app: enApp },
  nl: { app: nlApp },
  fr: { app: frApp },
  de: { app: deApp },
  es: { app: esApp },
  it: { app: itApp },
  ru: { app: ruApp },
  uk: { app: ukApp },
  hr: { app: hrApp },
  sl: { app: slApp },
  da: { app: daApp },
  sv: { app: svApp },
  nb: { app: nbApp },
  fi: { app: fiApp },
  tr: { app: trApp },
  "en-XA": { app: enXAApp },
  "ar-XB": { app: arXBApp },
} as const;

export interface CreateI18nOptions {
  lng?: string;
}

/** Typed t function over stable MessageId keys. */
export type TFunction = (
  key: MessageId,
  opts?: Record<string, unknown>,
) => string;

/** Bronze localization adapter returned by createI18n (narrow surface, hides raw i18next). */
export interface BronzeLocalizationAdapter {
  t: TFunction;
}

export async function createI18n(
  opts: CreateI18nOptions = {},
): Promise<BronzeLocalizationAdapter> {
  const instance: I18n = i18next
    .createInstance()
    .use(ICU)
    .use(initReactI18next);

  const chain = computeFallbackChain(opts.lng ?? "en");
  await instance.init({
    resources,
    lng: chain[0],
    fallbackLng: chain,
    keySeparator: false,
    ns: ["app"],
    defaultNS: "app",
    interpolation: { escapeValue: false },
    react: { useSuspense: false },
  });

  const adapter: BronzeLocalizationAdapter = {
    t: (key: MessageId, opts?: Record<string, unknown>) => {
      const r = instance.t(key, opts);
      return typeof r === "string" ? r : String(r);
    },
  };
  return adapter;
}

/** Back-compat alias; now resolves to the Bronze adapter (was raw i18next I18n). */
export type I18nInstance = BronzeLocalizationAdapter;
