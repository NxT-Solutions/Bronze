import { describe, expect, it } from "vitest";
import { windowAllows } from "./window-allows";

describe("windowAllows (QUE-001, SEC-002)", () => {
  it("denies import export backup on quick and allows them on library", () => {
    expect(windowAllows("quick", "import")).toBe(false);
    expect(windowAllows("quick", "export")).toBe(false);
    expect(windowAllows("quick", "backup")).toBe(false);
    expect(windowAllows("quick", "paginate")).toBe(false);
    expect(windowAllows("library", "paginate")).toBe(true);
    expect(windowAllows("library", "archive")).toBe(true);
    expect(windowAllows("library", "import")).toBe(true);
    expect(windowAllows("library", "export")).toBe(true);
    expect(windowAllows("library", "backup")).toBe(true);
  });
});
