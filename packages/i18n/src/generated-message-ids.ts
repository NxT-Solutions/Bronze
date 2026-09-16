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
  "panel.toolbar.overflow": "panel.toolbar.overflow",
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
  "library.backup.now": "library.backup.now",
  "library.restore.preview": "library.restore.preview",
  "library.export": "library.export",
  "library.import": "library.import",
  "library.state.empty": "library.state.empty",
  "library.state.loading": "library.state.loading",
  "library.state.readOnly": "library.state.readOnly",
  "copy.profile.label": "copy.profile.label",
  "copy.action.copy": "copy.action.copy",
  "copy.preview.label": "copy.preview.label",
  "settings.title": "settings.title",
  "settings.search.label": "settings.search.label",
  "settings.group.general": "settings.group.general",
  "settings.group.capture": "settings.group.capture",
  "settings.group.panel": "settings.group.panel",
  "settings.group.copy": "settings.group.copy",
  "settings.group.privacy": "settings.group.privacy",
  "settings.group.data": "settings.group.data",
  "settings.group.accessibility": "settings.group.accessibility",
  "settings.field.launchAtLogin": "settings.field.launchAtLogin",
  "settings.field.backupSchedule": "settings.field.backupSchedule",
  "settings.field.excludedBundleIds": "settings.field.excludedBundleIds",
  "settings.backup.daily": "settings.backup.daily",
  "settings.backup.weekly": "settings.backup.weekly",
  "settings.reset.field": "settings.reset.field",
  "settings.reset.group": "settings.reset.group",
  "settings.reset.all": "settings.reset.all",
  "settings.export.preview": "settings.export.preview",
  "settings.export.sensitive": "settings.export.sensitive",
  "settings.shortcuts.title": "settings.shortcuts.title",
  "settings.shortcuts.record": "settings.shortcuts.record",
  "settings.shortcuts.skipTest": "settings.shortcuts.skipTest",
  "settings.shortcuts.alternatives": "settings.shortcuts.alternatives",
  "settings.shortcuts.live": "settings.shortcuts.live",
  "settings.permission.title": "settings.permission.title",
  "settings.permission.retest": "settings.permission.retest",
  "settings.permission.inputMonitoring.why":
    "settings.permission.inputMonitoring.why",
  "settings.permission.inputMonitoring.alternative":
    "settings.permission.inputMonitoring.alternative",
  "settings.permission.accessibility.why":
    "settings.permission.accessibility.why",
  "settings.permission.accessibility.alternative":
    "settings.permission.accessibility.alternative",
  "settings.permission.launchAtLogin.why":
    "settings.permission.launchAtLogin.why",
  "settings.permission.launchAtLogin.alternative":
    "settings.permission.launchAtLogin.alternative",
  "settings.permission.automation.why": "settings.permission.automation.why",
  "settings.permission.automation.alternative":
    "settings.permission.automation.alternative",
  "settings.permission.screenRecording.why":
    "settings.permission.screenRecording.why",
  "settings.permission.screenRecording.alternative":
    "settings.permission.screenRecording.alternative",
  "settings.permission.screenRecording.notUsed":
    "settings.permission.screenRecording.notUsed",
  "settings.permission.selfTest.why": "settings.permission.selfTest.why",
  "settings.permission.selfTest.alternative":
    "settings.permission.selfTest.alternative",
  "settings.permission.composer.available":
    "settings.permission.composer.available",
} as const;

export type MessageId = keyof typeof MessageIds;

// Runtime list for cross-checks.
export const KNOWN_MESSAGE_IDS: readonly string[] = Object.keys(MessageIds);
