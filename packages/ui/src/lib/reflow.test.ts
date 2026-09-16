// @ts-nocheck
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  REFLOW_WIDTH_CSS_PX,
  TEXT_RESIZE_PERCENT,
  TWO_AXIS_SCROLL_ALLOWED,
  twoAxisScroll,
} from "@/lib/reflow";

const css = readFileSync(
  resolve(dirname(fileURLToPath(import.meta.url)), "../styles/globals.css"),
  "utf8",
);

describe("reflow (A11Y-003)", () => {
  it("forbids two-axis scroll at 320 CSS px and 200% text", () => {
    expect(REFLOW_WIDTH_CSS_PX).toBe(320);
    expect(TEXT_RESIZE_PERCENT).toBe(200);
    expect(TWO_AXIS_SCROLL_ALLOWED).toBe(false);
    expect(
      twoAxisScroll(
        { width: 320, height: 900 },
        { width: REFLOW_WIDTH_CSS_PX, height: 480 },
      ),
    ).toBe(false);
    expect(css).toMatch(/overflow-x:\s*hidden/);
    expect(css).toMatch(/flex-wrap:\s*wrap/);
  });
});
