import { describe, expect, it } from "vitest";
import {
  captureAlternativesOk,
  recorderSwallows,
  registerShortcut,
  SHORTCUT_ACTION_IDS,
  seedShortcutRegistry,
  standardChordAction,
} from "./shortcut-registry";

describe("shortcut registry (SET-002)", () => {
  it("seeds every action and keeps capture.selection as standardChord", () => {
    const seeded = seedShortcutRegistry();
    expect(seeded.map((row) => row.action)).toEqual([...SHORTCUT_ACTION_IDS]);
    expect(standardChordAction()).toBe("capture.selection");
  });

  it("retains the old chord when native registration fails", () => {
    const current = seedShortcutRegistry();
    const next = registerShortcut(
      current,
      { action: "capture.selection", chord: "Meta+K", enabled: true },
      false,
      { chord: true, menu: true, manual: true },
    );
    expect(next).toEqual(current);
  });

  it("refuses to disable chord menu and manual together and ignores IME", () => {
    expect(
      captureAlternativesOk({ chord: false, menu: false, manual: false }),
    ).toBe(false);
    const current = seedShortcutRegistry();
    const next = registerShortcut(
      current,
      { action: "capture.selection", chord: "", enabled: false },
      true,
      { chord: false, menu: false, manual: false },
    );
    expect(
      next.find((row) => row.action === "capture.selection")?.enabled,
    ).toBe(true);
    expect(
      recorderSwallows({
        isComposing: true,
        isVoiceOverReserved: false,
        isRepeat: false,
      }),
    ).toBe(false);
    expect(
      recorderSwallows({
        isComposing: false,
        isVoiceOverReserved: true,
        isRepeat: false,
      }),
    ).toBe(false);
  });
});
