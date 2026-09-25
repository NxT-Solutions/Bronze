import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyNoticeAuthorization,
  applyPermissionResult,
  loadNoticeAuthorization,
  NOTICE_REQUEST_COMMAND,
  NOTICE_STATUS_COMMAND,
  noticePillStatus,
  OPEN_SETTINGS_COMMAND,
  pillStatusForState,
  READ_COMMAND,
  RETEST_COMMAND,
  requestNoticeAuthorization,
  retestUsedPermission,
  shouldRevealNoticeAllow,
  shouldRevealNoticeSettings,
} from "./permission-health.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "settings.html"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

test("permission health lists independent rows and unused screen recording", () => {
  assert.match(html, /data-slot="permission-health"/);
  assert.match(html, /data-capability="inputMonitoring"/);
  assert.match(html, /data-capability="accessibility"/);
  assert.match(
    html,
    /data-capability="screenRecording"[^>]*data-usage="notUsed"/,
  );
  assert.match(html, /data-screen-recording-used="false"/);
  assert.equal(en["settings.permission.screenRecording.notUsed"], "Not used");
  assert.equal(
    en["settings.permission.composer.available"],
    "Manual composer remains available",
  );
  assert.equal(
    en["settings.permission.openSystemSettings"],
    "Open System Settings",
  );
  assert.match(html, /data-permission-retest="inputMonitoring"/);
  assert.match(html, /data-permission-retest="accessibility"/);
  assert.match(html, /data-permission-open-settings="inputMonitoring"/);
  assert.match(html, /data-capability="notifications"/);
  assert.match(html, /data-permission-allow-notifications/);
  assert.match(html, /data-permission-open-settings="notifications"/);
  assert.doesNotMatch(html, /data-permission-retest="notifications"/);
  assert.doesNotMatch(html, /data-permission-retest="screenRecording"/);
  assert.match(html, /permission-health\.mjs/);
  assert.equal(
    en["settings.permission.notifications.why"],
    "Needed for Notification Center banners",
  );
  assert.equal(
    en["settings.permission.notifications.alternative"],
    "The in-app notice still appears",
  );
  assert.equal(
    en["settings.permission.notifications.allow"],
    "Allow notifications",
  );
  assert.equal(en["settings.permission.status.unavailable"], "Unavailable");
  assert.equal(en["settings.permission.status.notRequested"], "Not requested");
  assert.equal(
    en["settings.permission.accessibility.usage"],
    "Bronze reads the current selection so a capture can use it.",
  );
  assert.equal(
    en["settings.permission.inputMonitoring.usage"],
    "Bronze watches the capture chord so a double-tap can add the selection.",
  );
  const plist = readFileSync(join(root, "../src-tauri/Info.plist"), "utf8");
  assert.match(plist, /NSAccessibilityUsageDescription/);
  assert.match(plist, /NSInputMonitoringUsageDescription/);
  assert.match(
    plist,
    /Bronze reads the current selection so a capture can use it\./,
  );
  assert.match(
    plist,
    /Bronze watches the capture chord so a double-tap can add the selection\./,
  );
  assert.doesNotMatch(plist, /NSScreenCaptureUsageDescription/);
  assert.doesNotMatch(plist, /NSCameraUsageDescription/);
  assert.doesNotMatch(plist, /NSMicrophoneUsageDescription/);
});

test("retest uses an injected request hook and never asks for screen recording", async () => {
  const commands = [];
  const denied = {
    input_monitoring: "denied",
    accessibility: "denied",
    listen_requested: true,
    accessibility_requested: true,
    screen_recording_requested: false,
  };
  const { revealSettings } = await retestUsedPermission(
    "accessibility",
    async (cmd) => {
      commands.push(cmd);
      return denied;
    },
  );
  assert.deepEqual(commands, [RETEST_COMMAND]);
  assert.equal(revealSettings, true);
  assert.notEqual(RETEST_COMMAND, OPEN_SETTINGS_COMMAND);
  await assert.rejects(
    () => retestUsedPermission("screenRecording", async () => denied),
    /capability_not_used/,
  );
  assert.equal(pillStatusForState("granted_unverified"), "granted");
  assert.equal(pillStatusForState("denied"), "denied");
  const pill = { dataset: {}, textContent: "Denied" };
  const card = { dataset: {}, querySelector: () => pill };
  const open = { hidden: true };
  applyPermissionResult(
    {
      querySelector(sel) {
        if (String(sel).includes("data-capability")) return card;
        if (String(sel).includes("open-settings")) return open;
        return pill;
      },
    },
    denied,
  );
  assert.equal(card.dataset.status, "denied");
  assert.equal(open.hidden, false);
  const granted = {
    input_monitoring: "granted_unverified",
    accessibility: "granted_unverified",
    listen_requested: true,
    accessibility_requested: true,
    screen_recording_requested: false,
  };
  const refreshed = await retestUsedPermission("accessibility", async (cmd) => {
    commands.push(cmd);
    return granted;
  });
  applyPermissionResult(
    {
      querySelector(sel) {
        if (String(sel).includes("data-capability")) return card;
        if (String(sel).includes("open-settings")) return open;
        return pill;
      },
    },
    refreshed.result,
  );
  assert.equal(card.dataset.status, "granted");
  assert.equal(pill.dataset.status, "granted");
  assert.equal(pill.textContent, "Granted");
  assert.equal(open.hidden, true);
  assert.equal(commands.filter((cmd) => cmd === RETEST_COMMAND).length, 2);
});

test("notice health loads status only and shows Allow while undetermined", async () => {
  const commands = [];
  const pill = { dataset: {}, textContent: "Unavailable", setAttribute() {} };
  const card = { dataset: {}, querySelector: () => pill };
  const allow = { hidden: true };
  const open = { hidden: true };
  const root = {
    querySelector(sel) {
      if (String(sel).includes('data-capability="notifications"')) return card;
      if (String(sel).includes("allow-notifications")) return allow;
      if (String(sel).includes("open-settings")) return open;
      return pill;
    },
  };
  const status = await loadNoticeAuthorization(root, async (cmd) => {
    commands.push(cmd);
    return "unavailable";
  });
  assert.deepEqual(commands, [NOTICE_STATUS_COMMAND]);
  assert.equal(status, "unavailable");
  assert.equal(noticePillStatus("unavailable"), "unavailable");
  assert.equal(noticePillStatus("not_requested"), "notRequested");
  assert.equal(shouldRevealNoticeAllow("not_requested"), true);
  assert.equal(shouldRevealNoticeAllow("unavailable"), false);
  assert.equal(shouldRevealNoticeSettings("denied"), true);
  applyNoticeAuthorization(root, "not_requested");
  assert.equal(card.dataset.status, "notRequested");
  assert.equal(allow.hidden, false);
  assert.equal(open.hidden, true);
  applyNoticeAuthorization(root, "denied");
  assert.equal(allow.hidden, true);
  assert.equal(open.hidden, false);
  const requested = await requestNoticeAuthorization(root, async (cmd) => {
    commands.push(cmd);
    return "healthy";
  });
  assert.deepEqual(commands, [NOTICE_STATUS_COMMAND, NOTICE_REQUEST_COMMAND]);
  assert.equal(requested, "healthy");
  assert.equal(card.dataset.status, "granted");
  assert.notEqual(NOTICE_REQUEST_COMMAND, RETEST_COMMAND);
});

test("opening permission health reads trust and does not prompt", async () => {
  const { bindPermissionHealth } = await import("./permission-health.mjs");
  const commands = [];
  const root = {
    querySelector() {
      return null;
    },
    querySelectorAll() {
      return [];
    },
  };
  bindPermissionHealth(root, async (cmd) => {
    commands.push(cmd);
    if (cmd === NOTICE_STATUS_COMMAND) return "not_requested";
    return {
      input_monitoring: "denied",
      accessibility: "denied",
      listen_requested: false,
      accessibility_requested: false,
      screen_recording_requested: false,
    };
  });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.ok(commands.includes(READ_COMMAND));
  assert.equal(commands.includes(RETEST_COMMAND), false);
  assert.equal(commands.includes(NOTICE_REQUEST_COMMAND), false);
});
