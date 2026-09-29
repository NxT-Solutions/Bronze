import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyNoticeAuthorization,
  applyPermissionResult,
  applyRetestFeedback,
  formatCodeIdentity,
  formatRunningCopy,
  loadNoticeAuthorization,
  NOTICE_REQUEST_COMMAND,
  NOTICE_STATUS_COMMAND,
  noticePillStatus,
  OPEN_SETTINGS_COMMAND,
  pillStatusForState,
  READ_COMMAND,
  RETEST_COMMAND,
  requestNoticeAuthorization,
  retestFeedbackKey,
  retestUsedPermission,
  runPermissionRetest,
  shouldRevealNoticeAllow,
  shouldRevealNoticeSettings,
  shouldShowIdentityDriftHint,
  shouldShowStaleCopyHint,
  shouldShowVersionReaddHint,
  SEEN_PERMISSION_VERSION_KEY,
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
  assert.match(html, /data-permission-running-copy/);
  assert.match(html, /data-permission-bundle-path/);
  assert.match(html, /data-permission-cdhash/);
  assert.match(html, /data-permission-stale-copy/);
  assert.match(html, /data-permission-exact-app/);
  assert.match(html, /data-permission-identity-drift/);
  assert.match(html, /data-permission-version-readd/);
  assert.match(html, /data-permission-retest-feedback/);
  assert.equal(
    en["settings.permission.runningCopy"],
    "This copy is {name} {version}.",
  );
  assert.equal(
    en["settings.permission.staleCopy"],
    "If System Settings already shows this app as on, turn that switch off, click Add, and choose the app revealed in Finder, then quit and reopen Bronze.",
  );
  assert.equal(
    en["settings.permission.exactApp"],
    "Add the exact file at the path above; another Bronze.app, bronze-desktop, or older copy will not grant this running app.",
  );
  assert.equal(
    en["settings.permission.identityDrift"],
    "An ad-hoc or unsigned update is a new binary identity, so add Accessibility and Input Monitoring again for the path and code identity above until releases share one Developer ID Application signature.",
  );
  assert.equal(
    en["settings.permission.versionReadd"],
    "After this version change, add Accessibility and Input Monitoring once more for this ad-hoc copy.",
  );
  assert.equal(
    en["settings.permission.title.info"],
    "Shows whether macOS has granted the permissions capture needs, with a way to retest or open System Settings, and explains when an ad-hoc update is a new binary identity.",
  );
  assert.equal(
    en["settings.permission.retestStillDenied"],
    "Still denied for this running copy: add the exact app at the path above in Privacy & Security, then quit and reopen Bronze.",
  );
  assert.equal(
    en["settings.permission.retestChecked"],
    "Checked for this running copy.",
  );
  assert.equal(
    en["settings.permission.retestFailed"],
    "Retest could not run: open System Settings and add the exact app at the path above.",
  );
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
  assert.equal(pillStatusForState("not_requested"), "notRequested");
  assert.equal(pillStatusForState("unknown"), "unavailable");
  const pill = { dataset: {}, textContent: "Denied", setAttribute() {} };
  const card = { dataset: {}, querySelector: () => pill };
  const open = { hidden: true };
  const copy = { hidden: true, textContent: "" };
  const pathEl = { hidden: true, textContent: "" };
  const identityEl = { hidden: true, textContent: "" };
  const stale = { hidden: true };
  const exact = { hidden: true };
  const drift = { hidden: true, textContent: "", setAttribute() {} };
  const versionReadd = { hidden: true, textContent: "", setAttribute() {} };
  const group = { hidden: true };
  applyPermissionResult(
    {
      querySelector(sel) {
        const query = String(sel);
        if (query.includes("data-capability")) return card;
        if (query.includes("open-settings")) return open;
        if (query.includes("running-copy")) return copy;
        if (query.includes("bundle-path")) return pathEl;
        if (query.includes("permission-cdhash")) return identityEl;
        if (query.includes("stale-copy")) return stale;
        if (query.includes("exact-app")) return exact;
        if (query.includes("identity-drift")) return drift;
        if (query.includes("version-readd")) return versionReadd;
        if (query.includes("permission-copy")) return group;
        return null;
      },
    },
    {
      ...denied,
      bundle_name: "Bronze",
      bundle_version: "0.1.1",
      bundle_path: "/Applications/Bronze.app",
      signature_kind: "adhoc",
      cdhash: "4356750bd341b36fe7f65ed5ecb29e994c25458b",
    },
  );
  assert.equal(card.dataset.status, "denied");
  assert.equal(open.hidden, false);
  assert.equal(copy.hidden, false);
  assert.equal(copy.textContent, "This copy is Bronze 0.1.1.");
  assert.equal(pathEl.textContent, "/Applications/Bronze.app");
  assert.equal(
    identityEl.textContent,
    "adhoc 4356750bd341b36fe7f65ed5ecb29e994c25458b",
  );
  assert.equal(stale.hidden, false);
  assert.equal(exact.hidden, false);
  assert.equal(drift.hidden, false);
  assert.equal(versionReadd.hidden, true);
  assert.equal(group.hidden, false);
  assert.equal(shouldShowStaleCopyHint(denied), true);
  assert.equal(
    shouldShowIdentityDriftHint({ ...denied, signature_kind: "adhoc" }),
    true,
  );
  assert.equal(
    shouldShowIdentityDriftHint({ ...denied, signature_kind: "signed" }),
    false,
  );
  const granted = {
    input_monitoring: "granted_unverified",
    accessibility: "granted_unverified",
    listen_requested: true,
    accessibility_requested: true,
    screen_recording_requested: false,
    bundle_name: "Bronze",
    bundle_version: "0.1.1",
    bundle_path: "/Applications/Bronze.app",
  };
  const refreshed = await retestUsedPermission("accessibility", async (cmd) => {
    commands.push(cmd);
    return granted;
  });
  applyPermissionResult(
    {
      querySelector(sel) {
        const query = String(sel);
        if (query.includes("data-capability")) return card;
        if (query.includes("open-settings")) return open;
        if (query.includes("running-copy")) return copy;
        if (query.includes("bundle-path")) return pathEl;
        if (query.includes("permission-cdhash")) return identityEl;
        if (query.includes("stale-copy")) return stale;
        if (query.includes("exact-app")) return exact;
        if (query.includes("permission-copy")) return group;
        return null;
      },
    },
    refreshed.result,
  );
  assert.equal(card.dataset.status, "granted");
  assert.equal(pill.dataset.status, "granted");
  assert.equal(pill.textContent, "Granted");
  assert.equal(open.hidden, true);
  assert.equal(stale.hidden, true);
  assert.equal(exact.hidden, true);
  assert.equal(commands.filter((cmd) => cmd === RETEST_COMMAND).length, 2);
  assert.equal(
    formatRunningCopy("This copy is {name} {version}.", "Bronze", "0.1.1"),
    "This copy is Bronze 0.1.1.",
  );
  assert.equal(
    formatCodeIdentity("adhoc", "4356750bd341b36fe7f65ed5ecb29e994c25458b"),
    "adhoc 4356750bd341b36fe7f65ed5ecb29e994c25458b",
  );
  applyPermissionResult(
    {
      querySelector(sel) {
        const query = String(sel);
        if (query.includes("data-capability")) return card;
        if (query.includes("open-settings")) return open;
        if (query.includes("running-copy")) return copy;
        if (query.includes("bundle-path")) return pathEl;
        if (query.includes("permission-cdhash")) return identityEl;
        if (query.includes("stale-copy")) return stale;
        if (query.includes("exact-app")) return exact;
        if (query.includes("permission-copy")) return group;
        return null;
      },
    },
    {
      input_monitoring: "granted_unverified",
      accessibility: "not_requested",
      listen_requested: false,
      accessibility_requested: false,
      screen_recording_requested: false,
      bundle_name: "Bronze",
      bundle_version: "0.2.0",
      bundle_path: "/Applications/Bronze.app",
      signature_kind: "adhoc",
      cdhash: "4356750bd341b36fe7f65ed5ecb29e994c25458b",
    },
  );
  assert.equal(card.dataset.status, "notRequested");
  assert.equal(pill.dataset.status, "notRequested");
  assert.equal(pill.textContent, "Not requested");
  assert.equal(open.hidden, false);
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

test("retest of a denied snapshot updates pills and does not auto-open Privacy", async () => {
  const commands = [];
  const pill = { dataset: {}, textContent: "Denied", setAttribute() {} };
  const card = { dataset: {}, querySelector: () => pill };
  const open = { hidden: true };
  const copy = { hidden: true, textContent: "" };
  const pathEl = { hidden: true, textContent: "" };
  const identityEl = { hidden: true, textContent: "" };
  const stale = { hidden: true };
  const exact = { hidden: true };
  const group = { hidden: true };
  const feedback = { hidden: true, textContent: "", setAttribute() {} };
  const root = {
    querySelector(sel) {
      const query = String(sel);
      if (query.includes("retest-feedback")) return feedback;
      if (query.includes("data-capability")) return card;
      if (query.includes("open-settings")) return open;
      if (query.includes("running-copy")) return copy;
      if (query.includes("bundle-path")) return pathEl;
      if (query.includes("permission-cdhash")) return identityEl;
      if (query.includes("stale-copy")) return stale;
      if (query.includes("exact-app")) return exact;
      if (query.includes("permission-copy")) return group;
      return null;
    },
  };
  const denied = {
    input_monitoring: "denied",
    accessibility: "denied",
    listen_requested: true,
    accessibility_requested: true,
    screen_recording_requested: false,
    bundle_name: "Bronze",
    bundle_version: "0.2.0",
    bundle_path: "/Applications/Bronze.app",
    signature_kind: "adhoc",
    cdhash: "4356750bd341b36fe7f65ed5ecb29e994c25458b",
  };
  const { result, revealSettings } = await runPermissionRetest(
    root,
    "accessibility",
    async (cmd, args) => {
      commands.push({ cmd, args });
      if (cmd === RETEST_COMMAND) {
        return denied;
      }
      return undefined;
    },
  );
  assert.equal(revealSettings, true);
  assert.equal(result.accessibility, "denied");
  assert.equal(card.dataset.status, "denied");
  assert.equal(pathEl.textContent, "/Applications/Bronze.app");
  assert.equal(exact.hidden, false);
  assert.equal(feedback.hidden, false);
  assert.equal(
    feedback.textContent,
    en["settings.permission.retestStillDenied"],
  );
  assert.equal(
    retestFeedbackKey("stillDenied"),
    "settings.permission.retestStillDenied",
  );
  applyRetestFeedback(root, "failed");
  assert.equal(feedback.textContent, en["settings.permission.retestFailed"]);
  assert.deepEqual(
    commands.map((row) => row.cmd),
    [RETEST_COMMAND],
  );
});

test("bind retest applies a denied snapshot and does not swallow the click", async () => {
  const { bindPermissionHealth } = await import("./permission-health.mjs");
  const commands = [];
  const clicks = [];
  const button = {
    getAttribute() {
      return "accessibility";
    },
    addEventListener(name, fn) {
      if (name === "click") {
        clicks.push(fn);
      }
    },
  };
  const pill = { dataset: {}, textContent: "Denied", setAttribute() {} };
  const card = { dataset: {}, querySelector: () => pill };
  const open = { hidden: true };
  const copy = { hidden: true, textContent: "" };
  const pathEl = { hidden: true, textContent: "" };
  const stale = { hidden: true };
  const exact = { hidden: true };
  const group = { hidden: true };
  const feedback = { hidden: true, textContent: "", setAttribute() {} };
  const root = {
    querySelector(sel) {
      const query = String(sel);
      if (query.includes("retest-feedback")) return feedback;
      if (query.includes("data-capability")) return card;
      if (query.includes("open-settings")) return open;
      if (query.includes("running-copy")) return copy;
      if (query.includes("bundle-path")) return pathEl;
      if (query.includes("stale-copy")) return stale;
      if (query.includes("exact-app")) return exact;
      if (query.includes("permission-copy")) return group;
      return null;
    },
    querySelectorAll(sel) {
      if (String(sel).includes("data-permission-retest")) {
        return [button];
      }
      return [];
    },
  };
  bindPermissionHealth(root, async (cmd, args) => {
    commands.push(cmd);
    if (cmd === NOTICE_STATUS_COMMAND) {
      return "not_requested";
    }
    if (cmd === RETEST_COMMAND) {
      return {
        input_monitoring: "denied",
        accessibility: "denied",
        listen_requested: true,
        accessibility_requested: true,
        screen_recording_requested: false,
        bundle_name: "Bronze",
        bundle_version: "0.2.0",
        bundle_path: "/Applications/Bronze.app",
      };
    }
    if (cmd === OPEN_SETTINGS_COMMAND) {
      assert.equal(args.capability, "accessibility");
    }
    return {
      input_monitoring: "not_requested",
      accessibility: "not_requested",
      listen_requested: false,
      accessibility_requested: false,
      screen_recording_requested: false,
      bundle_name: "Bronze",
      bundle_version: "0.2.0",
      bundle_path: "/Applications/Bronze.app",
    };
  });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(clicks.length, 1);
  await clicks[0]();
  assert.ok(commands.includes(RETEST_COMMAND));
  assert.equal(commands.includes(OPEN_SETTINGS_COMMAND), false);
  assert.equal(card.dataset.status, "denied");
  assert.equal(pathEl.textContent, "/Applications/Bronze.app");
  assert.equal(feedback.hidden, false);
});

test("ad-hoc version change asks for a one-time re-add", () => {
  const deniedAdhoc = {
    input_monitoring: "denied",
    accessibility: "denied",
    signature_kind: "adhoc",
    bundle_version: "0.2.1",
  };
  assert.equal(shouldShowIdentityDriftHint(deniedAdhoc), true);
  assert.equal(shouldShowVersionReaddHint(deniedAdhoc, "0.2.0"), true);
  assert.equal(shouldShowVersionReaddHint(deniedAdhoc, "0.2.1"), false);
  assert.equal(shouldShowVersionReaddHint(deniedAdhoc, ""), false);
  assert.equal(
    shouldShowIdentityDriftHint({
      ...deniedAdhoc,
      signature_kind: "signed",
    }),
    false,
  );
  const drift = { hidden: true, textContent: "", setAttribute() {} };
  const versionReadd = { hidden: true, textContent: "", setAttribute() {} };
  const storage = {
    data: { [SEEN_PERMISSION_VERSION_KEY]: "0.2.0" },
    getItem(key) {
      return this.data[key] ?? null;
    },
    setItem(key, value) {
      this.data[key] = value;
    },
  };
  applyPermissionResult(
    {
      querySelector(sel) {
        const query = String(sel);
        if (query.includes("identity-drift")) return drift;
        if (query.includes("version-readd")) return versionReadd;
        return null;
      },
    },
    {
      ...deniedAdhoc,
      bundle_name: "Bronze",
      bundle_path: "/Applications/Bronze.app",
      cdhash: "4356750bd341b36fe7f65ed5ecb29e994c25458b",
    },
    storage,
  );
  assert.equal(drift.hidden, false);
  assert.equal(versionReadd.hidden, false);
  assert.equal(storage.data[SEEN_PERMISSION_VERSION_KEY], "0.2.0");
  applyPermissionResult(
    {
      querySelector(sel) {
        const query = String(sel);
        if (query.includes("identity-drift")) return drift;
        if (query.includes("version-readd")) return versionReadd;
        return null;
      },
    },
    {
      input_monitoring: "granted_unverified",
      accessibility: "granted_unverified",
      signature_kind: "adhoc",
      bundle_name: "Bronze",
      bundle_version: "0.2.1",
      bundle_path: "/Applications/Bronze.app",
    },
    storage,
  );
  assert.equal(drift.hidden, true);
  assert.equal(versionReadd.hidden, true);
  assert.equal(storage.data[SEEN_PERMISSION_VERSION_KEY], "0.2.1");
});
