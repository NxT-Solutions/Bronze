import { describe, expect, it } from "vitest";
import {
  manualComposerAvailable,
  permissionHealthRows,
  retestUsedPermission,
  SCREEN_RECORDING_USED,
  shouldRevealSystemSettings,
  USED_PERMISSION_COMMANDS,
} from "./permission-health";

describe("permission health (SET-003, SET-004, CAP-003)", () => {
  it("keeps capabilities independent and marks screen recording unused", () => {
    const rows = permissionHealthRows({
      inputMonitoring: "denied",
      accessibility: "granted_unverified",
      selfTest: "unknown",
    });
    expect(
      rows.find((row) => row.capability === "inputMonitoring")?.state,
    ).toBe("denied");
    expect(rows.find((row) => row.capability === "accessibility")?.state).toBe(
      "granted_unverified",
    );
    expect(
      rows.find((row) => row.capability === "screenRecording")?.usage,
    ).toBe("notUsed");
    expect(SCREEN_RECORDING_USED).toBe(false);
    expect(manualComposerAvailable()).toBe(true);
  });

  it("retest invokes the request hook and reveals settings after denial", async () => {
    const denied = {
      input_monitoring: "denied",
      accessibility: "denied",
      listen_requested: true,
      accessibility_requested: true,
      screen_recording_requested: false,
    };
    const commands: string[] = [];
    const result = await retestUsedPermission(
      "inputMonitoring",
      async (cmd) => {
        commands.push(cmd);
        return denied;
      },
    );
    expect(commands).toEqual([USED_PERMISSION_COMMANDS.retest]);
    expect(result.revealSettings).toBe(true);
    expect(shouldRevealSystemSettings("screenRecording", denied)).toBe(false);
    await expect(
      retestUsedPermission("screenRecording", async () => denied),
    ).rejects.toThrow("capability_not_used");
  });
});
