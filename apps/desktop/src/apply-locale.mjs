import { tauriInvoke } from "./tauri-bridge.mjs";

export const HAND_TEST_DEFAULT_LOCALE = "en";

export function resolveUiLocale(requested) {
  if (requested === "en-XA" || requested === "ar-XB") {
    return requested;
  }
  return HAND_TEST_DEFAULT_LOCALE;
}

export function catalogHasRealSpaces(value) {
  return typeof value === "string" && /\s/.test(value);
}

export function applyHandTestLocale(
  root = document,
  locale = HAND_TEST_DEFAULT_LOCALE,
) {
  const resolved = resolveUiLocale(locale);
  const html = root.documentElement ?? root;
  html.lang = resolved === "ar-XB" ? "ar" : "en";
  html.dir = resolved === "ar-XB" ? "rtl" : "ltr";
  return resolved;
}

export async function bindHandTestLocale(
  root = document,
  invokeFn = tauriInvoke,
) {
  let requested = HAND_TEST_DEFAULT_LOCALE;
  try {
    requested = await invokeFn("ui_locale");
  } catch {
    requested = HAND_TEST_DEFAULT_LOCALE;
  }
  return applyHandTestLocale(root, requested);
}

if (globalThis.document?.readyState) {
  bindHandTestLocale();
}
