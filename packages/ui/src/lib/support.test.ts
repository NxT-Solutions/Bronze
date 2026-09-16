import { describe, expect, it } from "vitest";
import {
  AUTOMATIC_UPLOAD,
  exportBundle,
  HUMAN_GATES,
  previewBundle,
} from "@/lib/support";

describe("support bundle (SEC-006)", () => {
  it("requires preview ack and lists human gates", () => {
    expect(AUTOMATIC_UPLOAD).toBe(false);
    const preview = previewBundle(2);
    expect(preview.text).toContain("3.9");
    expect(HUMAN_GATES).toContain("9.3");
    expect(() => exportBundle(preview, false)).toThrow("PreviewRequired");
    expect(exportBundle(preview, true).eventCount).toBe(2);
  });
});
