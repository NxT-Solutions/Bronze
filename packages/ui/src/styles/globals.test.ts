// @ts-nocheck
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const css = readFileSync(
  resolve(dirname(fileURLToPath(import.meta.url)), "globals.css"),
  "utf8",
);

describe("globals.css a11y hooks (A11Y-003)", () => {
  it("keeps Reduce Motion and Increase Contrast media hooks and !important overrides", () => {
    expect(css).toMatch(/prefers-reduced-motion:\s*reduce/);
    expect(css).toMatch(/prefers-contrast:\s*more/);
    // A11Y-003: !important strengthens overrides vs Tailwind (see ADR-013, 1.7)
    expect(css).toMatch(/transition:\s*none\s*!important/);
    expect(css).toMatch(/animation:\s*none\s*!important/);
    expect(css).toMatch(/scroll-behavior:\s*auto\s*!important/);
    expect(css).toMatch(/border-width:\s*2px\s*!important/);
  });
});
