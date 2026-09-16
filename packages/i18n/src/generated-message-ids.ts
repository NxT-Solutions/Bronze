// Seeded from packages/i18n/locales/en/app.json semantic keys.
// Used for future compile-time key safety (no string literals for keys outside this).

export const MessageIds = {
  "app.name": "app.name",
  "queue.count": "queue.count",
  "capture.source": "capture.source",
  "export.preview.secretBodies": "export.preview.secretBodies",
  "menu.status.capture": "menu.status.capture",
  "menu.status.newNote": "menu.status.newNote",
  "menu.status.show": "menu.status.show",
  "menu.status.settings": "menu.status.settings",
  "menu.status.quit": "menu.status.quit",
  "panel.quick.title": "panel.quick.title",
  "capture.announce.saved": "capture.announce.saved",
  "composer.add.label": "composer.add.label",
  "composer.add.submit": "composer.add.submit",
  "composer.add.error": "composer.add.error",
  "queue.item.moveUp": "queue.item.moveUp",
  "queue.item.moveDown": "queue.item.moveDown",
  "queue.item.complete": "queue.item.complete",
  "queue.item.skip": "queue.item.skip",
  "queue.item.trash": "queue.item.trash",
  "queue.item.edit": "queue.item.edit",
} as const;

export type MessageId = keyof typeof MessageIds;

// Runtime list for cross-checks.
export const KNOWN_MESSAGE_IDS: readonly string[] = Object.keys(MessageIds);
