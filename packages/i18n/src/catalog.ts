/** Native menu/InfoPlist share the WebView glossary (I18N-001/002, G-06). */

export const ADVERTISED_LOCALES = [
  "en",
  "nl",
  "fr",
  "de",
  "es",
  "it",
  "en-XA",
  "ar-XB",
] as const;

export const INFOPLIST_GLOSSARY = [
  ["CFBundleDisplayName", "app.name"],
  ["CFBundleName", "app.name"],
  [
    "NSAccessibilityUsageDescription",
    "settings.permission.accessibility.usage",
  ],
  [
    "NSInputMonitoringUsageDescription",
    "settings.permission.inputMonitoring.usage",
  ],
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

export const PANEL_CHROME_KEYS = [
  "panel.quick.title",
  "panel.section.active",
  "panel.empty",
  "chrome.skip.toContent",
  "composer.add.label",
  "composer.placeholder",
  "capture.source",
  "queue.item.showMore",
  "queue.item.showLess",
  "queue.item.title",
  "library.heading.items",
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
