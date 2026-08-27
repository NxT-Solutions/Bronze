// Seeded from packages/i18n/locales/en/app.json semantic keys.
// Used for future compile-time key safety (no string literals for keys outside this).

export const MessageIds = {
  "app.name": "app.name",
  "queue.count": "queue.count",
  "capture.source": "capture.source",
} as const;

export type MessageId = keyof typeof MessageIds;

// Runtime list for cross-checks.
export const KNOWN_MESSAGE_IDS: readonly string[] = Object.keys(MessageIds);
