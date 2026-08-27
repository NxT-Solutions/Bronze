// Cached Intl formatters by locale. Expand as needed by consumers.
const cache = new Map<
  string,
  | Intl.NumberFormat
  | Intl.DateTimeFormat
  | Intl.PluralRules
  | Intl.RelativeTimeFormat
  | Intl.ListFormat
  | Intl.Collator
  | Intl.DisplayNames
  | Intl.Segmenter
>();

function cacheKey(prefix: string, locale: string, opts?: object): string {
  const o = opts || {};
  const sorted = JSON.stringify(o, Object.keys(o).sort());
  return `${prefix}:${locale}:${sorted}`;
}

export function getNumberFormat(
  locale: string,
  opts?: Intl.NumberFormatOptions,
): Intl.NumberFormat {
  const key = cacheKey("nf", locale, opts);
  if (!cache.has(key)) {
    cache.set(key, new Intl.NumberFormat(locale, opts));
  }
  return cache.get(key) as Intl.NumberFormat;
}

export function getDateTimeFormat(
  locale: string,
  opts?: Intl.DateTimeFormatOptions,
): Intl.DateTimeFormat {
  const key = cacheKey("dt", locale, opts);
  if (!cache.has(key)) {
    cache.set(key, new Intl.DateTimeFormat(locale, opts));
  }
  return cache.get(key) as Intl.DateTimeFormat;
}

export function getPluralRules(
  locale: string,
  opts?: Intl.PluralRulesOptions,
): Intl.PluralRules {
  const key = cacheKey("pr", locale, opts);
  if (!cache.has(key)) {
    cache.set(key, new Intl.PluralRules(locale, opts));
  }
  return cache.get(key) as Intl.PluralRules;
}

/** Cached RelativeTimeFormat (e.g. "2 days ago"). */
export function getRelativeTimeFormat(
  locale: string,
  opts?: Intl.RelativeTimeFormatOptions,
): Intl.RelativeTimeFormat {
  const key = cacheKey("rt", locale, opts);
  if (!cache.has(key)) {
    cache.set(key, new Intl.RelativeTimeFormat(locale, opts));
  }
  return cache.get(key) as Intl.RelativeTimeFormat;
}

export function getListFormat(
  locale: string,
  opts?: Intl.ListFormatOptions,
): Intl.ListFormat {
  const key = cacheKey("lf", locale, opts);
  if (!cache.has(key)) {
    cache.set(key, new Intl.ListFormat(locale, opts));
  }
  return cache.get(key) as Intl.ListFormat;
}

export function getCollator(
  locale: string,
  opts?: Intl.CollatorOptions,
): Intl.Collator {
  const key = cacheKey("co", locale, opts);
  if (!cache.has(key)) {
    cache.set(key, new Intl.Collator(locale, opts));
  }
  return cache.get(key) as Intl.Collator;
}

export function getDisplayNames(
  locale: string,
  opts?: Intl.DisplayNamesOptions,
): Intl.DisplayNames {
  const dnOpts = {
    type: "language",
    ...(opts ?? {}),
  } as Intl.DisplayNamesOptions;
  const key = cacheKey("dn", locale, dnOpts);
  if (!cache.has(key)) {
    cache.set(key, new Intl.DisplayNames(locale, dnOpts));
  }
  return cache.get(key) as Intl.DisplayNames;
}

export function getSegmenter(
  locale: string,
  opts?: Intl.SegmenterOptions,
): Intl.Segmenter {
  const key = cacheKey("sg", locale, opts);
  if (!cache.has(key)) {
    cache.set(key, new Intl.Segmenter(locale, opts));
  }
  return cache.get(key) as Intl.Segmenter;
}
