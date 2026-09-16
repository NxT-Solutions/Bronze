import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  OPEN_SETTINGS_COMMAND,
  RETEST_COMMAND,
  retestUsedPermission,
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
  assert.doesNotMatch(html, /data-permission-retest="screenRecording"/);
  assert.match(html, /permission-health\.mjs/);
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
});
