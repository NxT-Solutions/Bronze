import type { i18n as I18n } from "i18next";
import i18next from "i18next";
import ICU from "i18next-icu";
import { initReactI18next } from "react-i18next";
import arXBApp from "../locales/ar-XB/app.json" with { type: "json" };
import enApp from "../locales/en/app.json" with { type: "json" };
import enXAApp from "../locales/en-XA/app.json" with { type: "json" };
import type { MessageId } from "./generated-message-ids";
import { computeFallbackChain } from "./locale";

const resources = {
  en: { app: enApp },
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
