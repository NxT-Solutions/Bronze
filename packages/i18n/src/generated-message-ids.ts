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
  "panel.section.active": "panel.section.active",
  "library.title": "library.title",
  "library.nav.archive": "library.nav.archive",
  "library.nav.trash": "library.nav.trash",
  "library.nav.search": "library.nav.search",
  "library.search.label": "library.search.label",
  "library.archive": "library.archive",
  "library.paginate": "library.paginate",
  "library.state.empty": "library.state.empty",
  "library.state.loading": "library.state.loading",
  "library.state.readOnly": "library.state.readOnly",
  "copy.profile.label": "copy.profile.label",
  "copy.action.copy": "copy.action.copy",
  "copy.preview.label": "copy.preview.label",
} as const;

export type MessageId = keyof typeof MessageIds;

// Runtime list for cross-checks.
export const KNOWN_MESSAGE_IDS: readonly string[] = Object.keys(MessageIds);
