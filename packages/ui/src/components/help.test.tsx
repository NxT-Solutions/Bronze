import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Help } from "@/components/help";
import { AUTOMATIC_UPLOAD } from "@/lib/support";

const labels = {
  title: "Help",
  about: "About Bronze",
  preview: "Diagnostics preview",
  exportBundle: "Export support bundle",
  humanGates:
    "Human validation remains backlog: stories 3.9, 3.10, 5.5, and 9.3.",
  noUpload: "Bronze never uploads diagnostics automatically.",
};

describe("Help (SUP-001/002, SEC-006)", () => {
  it("previews redacted diagnostics without automatic upload", async () => {
    const user = userEvent.setup();
    const onExport = vi.fn();
    const { container } = render(
      <Help
        labels={labels}
        preview="events=1 limits=3.9,3.10,5.5,9.3"
        onExport={onExport}
      />,
    );
    expect(AUTOMATIC_UPLOAD).toBe(false);
    expect(
      container.querySelector("[data-automatic-upload='false']"),
    ).toBeTruthy();
    expect(screen.getByText("3.9")).toBeTruthy();
    expect(screen.getByText("9.3")).toBeTruthy();
    expect(
      container.querySelector("[data-diagnostics-preview]")?.textContent,
    ).toContain("events=1");
    await user.click(
      screen.getByRole("button", { name: "Export support bundle" }),
    );
    expect(onExport).toHaveBeenCalledTimes(1);
  });
});
