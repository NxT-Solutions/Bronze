import { tauriEmit, tauriInvoke, tauriListen } from "./tauri-bridge.mjs";

export const UI_LOCALE_EVENT = "ui-locale-changed";
export const LOCALE_APPLIED_EVENT = "bronze:locale-changed";

export const HAND_TEST_DEFAULT_LOCALE = "en";

export const SHIPPED_UI_LOCALES = [
  "en",
  "nl",
  "fr",
  "de",
  "es",
  "it",
  "en-XA",
  "ar-XB",
];

let lastCatalog = {
  locale: HAND_TEST_DEFAULT_LOCALE,
  htmlLang: "en",
  dir: "ltr",
  catalogAvailable: false,
  messages: {},
};

export function catalogMessage(key) {
  const value = lastCatalog.messages?.[key];
  return typeof value === "string" ? value : "";
}

export function resolveUiLocale(requested) {
  if (requested === "en-XA" || requested === "ar-XB") {
    return requested;
  }
  if (SHIPPED_UI_LOCALES.includes(requested)) {
    return requested;
  }
  const primary = String(requested ?? "")
    .trim()
    .split(/[-_]/)[0];
  if (SHIPPED_UI_LOCALES.includes(primary)) {
    return primary;
  }
  return HAND_TEST_DEFAULT_LOCALE;
}

export function htmlLangFor(locale) {
  if (locale === "ar-XB") {
    return "ar";
  }
  if (locale === "en-XA") {
    return "en";
  }
  return resolveUiLocale(locale);
}

export function catalogHasRealSpaces(value) {
  return typeof value === "string" && /\s/.test(value);
}

export function isIcuMessage(value) {
  return /,\s*(plural|select|selectordinal)\s*,/i.test(value);
}

export function formatQueueCount(count, template) {
  const fallback = "{count, plural, =0 {0 items} one {1 item} other {# items}}";
  const src =
    typeof template === "string" && isIcuMessage(template)
      ? template
      : fallback;
  const n = Number(count);
  const zero = src.match(/=0\s*\{([^}]*)\}/)?.[1];
  const one = src.match(/\bone\s*\{([^}]*)\}/)?.[1];
  const other = src.match(/\bother\s*\{([^}]*)\}/)?.[1];
  let chosen = other ?? `${n} items`;
  if (n === 0 && zero !== undefined) {
    chosen = zero;
  } else if (n === 1 && one !== undefined) {
    chosen = one;
  }
  return chosen.replaceAll("#", String(n));
}

function applyCatalogMessages(scope, messages) {
  if (!scope?.querySelectorAll || !messages) {
    return;
  }
  for (const el of scope.querySelectorAll("[data-i18n]")) {
    if (el.closest?.("[data-slot=title],[data-slot=body]")) {
      continue;
    }
    const key = el.getAttribute("data-i18n");
    const value = messages[key];
    if (typeof value !== "string" || isIcuMessage(value)) {
      continue;
    }
    el.textContent = value;
  }
  for (const el of scope.querySelectorAll("[data-i18n-placeholder]")) {
    const key = el.getAttribute("data-i18n-placeholder");
    const value = messages[key];
    if (typeof value === "string" && !isIcuMessage(value)) {
      el.setAttribute("placeholder", value);
      el.setAttribute("aria-placeholder", value);
    }
  }
  for (const el of scope.querySelectorAll("[data-i18n-title]")) {
    const key = el.getAttribute("data-i18n-title");
    const value = messages[key];
    if (typeof value === "string" && !isIcuMessage(value)) {
      el.setAttribute("title", value);
    }
  }
  for (const el of scope.querySelectorAll("[data-i18n-aria-label]")) {
    const key = el.getAttribute("data-i18n-aria-label");
    const value = messages[key];
    if (typeof value === "string" && !isIcuMessage(value)) {
      el.setAttribute("aria-label", value);
    }
  }
  for (const template of scope.querySelectorAll?.("template") ?? []) {
    applyCatalogMessages(template.content, messages);
  }
}

export function applyHandTestLocale(
  root = document,
  locale = HAND_TEST_DEFAULT_LOCALE,
  catalog,
) {
  const resolved = catalog?.locale
    ? resolveUiLocale(catalog.locale)
    : resolveUiLocale(locale);
  const html = root.documentElement ?? root;
  const htmlLang = catalog?.htmlLang || htmlLangFor(resolved);
  const dir = catalog?.dir || (resolved === "ar-XB" ? "rtl" : "ltr");
  html.lang = htmlLang;
  html.dir = dir;
  const panel = root.getElementById?.("quick-panel");
  if (panel) {
    panel.lang = htmlLang;
    panel.dir = dir;
  }
  if (catalog?.messages) {
    lastCatalog = {
      locale: resolved,
      htmlLang,
      dir,
      catalogAvailable: Boolean(catalog.catalogAvailable),
      messages: catalog.messages,
    };
    applyCatalogMessages(root, catalog.messages);
    let titleKey = "panel.quick.title";
    if (root.getElementById?.("settings")) {
      titleKey = "settings.title";
    } else if (root.getElementById?.("library")) {
      titleKey = "library.title";
    } else if (root.getElementById?.("help")) {
      titleKey = "help.title";
    }
    const title = catalog.messages[titleKey];
    const titleEl = root.querySelector?.("title");
    if (title && titleEl) {
      titleEl.textContent = title;
    }
  }
  return resolved;
}

export function notifyLocaleApplied(root = document) {
  if (typeof root.dispatchEvent !== "function") {
    return;
  }
  root.dispatchEvent(new CustomEvent(LOCALE_APPLIED_EVENT, { bubbles: true }));
}

export function emitUiLocaleChanged(payload = {}) {
  try {
    const channel = new BroadcastChannel(UI_LOCALE_EVENT);
    channel.postMessage(payload);
    channel.close();
  } catch {
    // BroadcastChannel is absent in some test runtimes.
  }
  return tauriEmit(UI_LOCALE_EVENT, payload);
}

export function listenUiLocaleChanged(handler) {
  let channel;
  try {
    channel = new BroadcastChannel(UI_LOCALE_EVENT);
    channel.onmessage = (event) => {
      handler(event.data ?? {});
    };
  } catch {
    channel = null;
  }
  tauriListen(UI_LOCALE_EVENT, (event) => {
    handler(event?.payload ?? event ?? {});
  });
  return () => {
    channel?.close();
  };
}

let localeApplyInFlight = null;

export async function bindHandTestLocale(
  root = document,
  invokeFn = tauriInvoke,
) {
  let catalog;
  let requested = HAND_TEST_DEFAULT_LOCALE;
  try {
    catalog = await invokeFn("ui_catalog");
    requested = catalog?.locale ?? HAND_TEST_DEFAULT_LOCALE;
  } catch {
    try {
      requested = await invokeFn("ui_locale");
    } catch {
      requested = HAND_TEST_DEFAULT_LOCALE;
    }
  }
  const resolved = applyHandTestLocale(root, requested, catalog);
  notifyLocaleApplied(root);
  return resolved;
}

export function listenForLocaleChanges(
  root = document,
  invokeFn = tauriInvoke,
) {
  return listenUiLocaleChanged(() => {
    if (localeApplyInFlight) {
      return localeApplyInFlight;
    }
    localeApplyInFlight = bindHandTestLocale(root, invokeFn).finally(() => {
      localeApplyInFlight = null;
    });
    return localeApplyInFlight;
  });
}

if (globalThis.document?.readyState) {
  bindHandTestLocale();
  listenForLocaleChanges();
}
