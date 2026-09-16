import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it } from "vitest";
import { ShortcutRecorder } from "@/components/shortcut-recorder";
import { SHORTCUT_ACTION_IDS } from "@/lib/shortcut-registry";

const labels = {
  title: "Shortcuts",
  record: "Record shortcut",
  skipTest: "Skip test",
  alternatives: "Keep a menu or manual path",
  live: "Listening",
};

describe("ShortcutRecorder (SET-002, A11Y-001)", () => {
  it("lists every action and skippable test without swallowing IME", async () => {
    const user = userEvent.setup();
    const { container } = render(<ShortcutRecorder labels={labels} />);
    for (const action of SHORTCUT_ACTION_IDS) {
      expect(container.querySelector(`[data-action='${action}']`)).toBeTruthy();
    }
    expect(container.querySelector("[data-standard-chord]")).toHaveAttribute(
      "data-standard-chord",
      "capture.selection",
    );
    await user.click(screen.getByRole("button", { name: "Skip test" }));
    expect(screen.getByRole("status")).toHaveTextContent("skipped");
    const record = screen.getByLabelText("Record shortcut");
    await user.type(record, "k");
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
