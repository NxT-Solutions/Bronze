import { describe, expect, it } from "vitest";
import {
  applyDisplaySnapshot,
  readDisplaySnapshot,
} from "./apply-display-snapshot";

describe("applyDisplaySnapshot", () => {
  it("applies one typed snapshot without restart and only strengthens", () => {
    const root = document.createElement("html");
    applyDisplaySnapshot(root, {
      reduceMotion: true,
      reduceTransparency: false,
      increaseContrast: true,
      differentiateWithoutColor: true,
    });
    expect(readDisplaySnapshot(root)).toEqual({
      reduceMotion: true,
      reduceTransparency: false,
      increaseContrast: true,
      differentiateWithoutColor: true,
    });
    applyDisplaySnapshot(root, {
      reduceMotion: false,
      reduceTransparency: false,
      increaseContrast: false,
      differentiateWithoutColor: false,
    });
    expect(root.hasAttribute("data-reduce-motion")).toBe(false);
  });
});
