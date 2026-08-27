import type { i18n as I18n } from "i18next";
import i18next from "i18next";
import ICU from "i18next-icu";
import { initReactI18next } from "react-i18next";
import arXBApp from "../locales/ar-XB/app.json" with { type: "json" };
import enApp from "../locales/en/app.json" with { type: "json" };
import enXAApp from "../locales/en-XA/app.json" with { type: "json" };
import { computeFallbackChain } from "./locale";

const resources = {
  en: { app: enApp },
  "en-XA": { app: enXAApp },
  "ar-XB": { app: arXBApp },
} as const;

export interface CreateI18nOptions {
  lng?: string;
}

export async function createI18n(opts: CreateI18nOptions = {}): Promise<I18n> {
  const instance = i18next.createInstance().use(ICU).use(initReactI18next);

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

  return instance;
}

export type I18nInstance = I18n;
