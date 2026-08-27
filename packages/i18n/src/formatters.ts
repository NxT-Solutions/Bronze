// Cached Intl formatters by locale. Expand as needed by consumers.
const cache = new Map<
  string,
  Intl.NumberFormat | Intl.DateTimeFormat | Intl.PluralRules
>();

export function getNumberFormat(
  locale: string,
  opts?: Intl.NumberFormatOptions,
): Intl.NumberFormat {
  const key = `nf:${locale}:${JSON.stringify(opts || {})}`;
  if (!cache.has(key)) {
    cache.set(key, new Intl.NumberFormat(locale, opts));
  }
  return cache.get(key) as Intl.NumberFormat;
}

export function getDateTimeFormat(
  locale: string,
  opts?: Intl.DateTimeFormatOptions,
): Intl.DateTimeFormat {
  const key = `dt:${locale}:${JSON.stringify(opts || {})}`;
  if (!cache.has(key)) {
    cache.set(key, new Intl.DateTimeFormat(locale, opts));
  }
  return cache.get(key) as Intl.DateTimeFormat;
}

export function getPluralRules(
  locale: string,
  opts?: Intl.PluralRulesOptions,
): Intl.PluralRules {
  const key = `pr:${locale}:${JSON.stringify(opts || {})}`;
  if (!cache.has(key)) {
    cache.set(key, new Intl.PluralRules(locale, opts));
  }
  return cache.get(key) as Intl.PluralRules;
}

// t wrapper placeholder; real t from i18n instance
export type TFunction = (key: string, opts?: Record<string, unknown>) => string;
