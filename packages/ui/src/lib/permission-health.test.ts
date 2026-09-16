import { describe, expect, it } from "vitest";
import {
  manualComposerAvailable,
  permissionHealthRows,
  SCREEN_RECORDING_USED,
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
});
