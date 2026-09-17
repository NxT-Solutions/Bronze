// @ts-nocheck
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  blockVars,
  CONCEPT_PNG_IS_PIXEL_SPEC,
  contrastRatio,
  MIN_ENHANCED_TEXT_CONTRAST,
  MIN_NON_TEXT_CONTRAST,
  MIN_NORMAL_TEXT_CONTRAST,
  PILL_PAIRS,
  TEXT_PAIRS,
} from "@/lib/contrast";

const here = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(resolve(here, "../styles/globals.css"), "utf8");
const chrome = readFileSync(
  resolve(here, "../../../../apps/desktop/src/chrome.css"),
  "utf8",
);

describe("bronze tokens contrast (A11Y-003)", () => {
  it("samples light and dark normal text at ≥4.5:1", () => {
    const light = blockVars(css, ":root");
    const dark = blockVars(css, ".dark");
    for (const theme of [light, dark]) {
      for (const [fg, bg] of TEXT_PAIRS) {
        expect(contrastRatio(theme[fg], theme[bg])).toBeGreaterThanOrEqual(
          MIN_NORMAL_TEXT_CONTRAST,
        );
      }
    }
    expect(CONCEPT_PNG_IS_PIXEL_SPEC).toBe(false);
    expect(css.toLowerCase()).not.toMatch(/copper|cooper/);
    expect(css).toMatch(/from DESIGN\.md/);
    const parchment = blockVars(chrome, ":root");
    for (const [fg, bg] of TEXT_PAIRS) {
      expect(
        contrastRatio(parchment[fg], parchment[bg]),
      ).toBeGreaterThanOrEqual(MIN_NORMAL_TEXT_CONTRAST);
    }
    for (const theme of [light, dark, parchment]) {
      for (const [fg, bg] of PILL_PAIRS) {
        expect(contrastRatio(theme[fg], theme[bg])).toBeGreaterThanOrEqual(
          MIN_NORMAL_TEXT_CONTRAST,
        );
      }
    }
    expect(chrome.toLowerCase()).not.toMatch(/copper|cooper/);
    expect(parchment.border).toBe("#e4e4e7");
    expect(parchment.success).toBe("#2f6f4f");
    expect(MIN_NORMAL_TEXT_CONTRAST).toBe(4.5);
    expect(MIN_ENHANCED_TEXT_CONTRAST).toBe(7);
    expect(MIN_NON_TEXT_CONTRAST).toBe(3);
  });
});
