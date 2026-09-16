// @ts-nocheck
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  blockVars,
  CONCEPT_PNG_IS_PIXEL_SPEC,
  contrastRatio,
  MIN_NORMAL_TEXT_CONTRAST,
  TEXT_PAIRS,
} from "@/lib/contrast";

const css = readFileSync(
  resolve(dirname(fileURLToPath(import.meta.url)), "../styles/globals.css"),
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
  });
});
