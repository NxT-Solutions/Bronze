/** Native + WebView glossary keys (I18N-001/002/003/004, G-06). */

export const ADVERTISED_LOCALES = ["en", "en-XA", "ar-XB"] as const;

export const INFOPLIST_GLOSSARY = [
  ["CFBundleDisplayName", "app.name"],
  ["CFBundleName", "app.name"],
] as const;

export const NATIVE_GLOSSARY_KEYS = [
  "app.name",
  "menu.status.capture",
  "menu.status.newNote",
  "menu.status.show",
  "menu.status.settings",
  "menu.status.quit",
  "panel.quick.title",
] as const;

export function chromeDir(locale: string): "ltr" | "rtl" {
  const primary = locale.split("-")[0];
  return primary === "ar" ||
    primary === "he" ||
    primary === "fa" ||
    primary === "ur"
    ? "rtl"
    : "ltr";
}

export function cardLangDir(contentLanguage: string | undefined): {
  lang: string;
  dir: "auto";
} {
  return {
    lang:
      contentLanguage && contentLanguage.length > 0 ? contentLanguage : "und",
    dir: "auto",
  };
}
