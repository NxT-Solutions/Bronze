import { render, screen } from "@testing-library/react";
import axe from "axe-core";
import { describe, expect, it } from "vitest";
import { PermissionHealth } from "@/components/permission-health";

const labels = {
  title: "Permission health",
  retest: "Retest",
  notUsed: "Not used",
  composer: "Manual composer remains available",
};

const why = {
  inputMonitoring: "Needed for the global capture chord",
  accessibility: "Needed to read the current selection",
  launchAtLogin: "Optional start at login",
  automation: "Not required in this release",
  screenRecording: "Bronze does not record the screen",
  selfTest: "Checks the capture pipeline without content",
};

const alternative = {
  inputMonitoring: "Use the menu or composer",
  accessibility: "Paste or type into the composer",
  launchAtLogin: "Open Bronze from the menu",
  automation: "No alternative required",
  screenRecording: "No screen permission is requested",
  selfTest: "Retry later from this list",
};

describe("PermissionHealth (SET-003, CAP-003)", () => {
  it("shows independent rows and unused screen recording", async () => {
    const { container } = render(
      <PermissionHealth labels={labels} why={why} alternative={alternative} />,
    );
    expect(
      screen.getByRole("heading", { name: "Permission health" }),
    ).toBeTruthy();
    expect(
      container.querySelector("[data-capability='screenRecording']"),
    ).toHaveAttribute("data-usage", "notUsed");
    expect(screen.getByText("Not used")).toBeTruthy();
    expect(screen.getByText("Manual composer remains available")).toBeTruthy();
    expect(
      container.querySelector("[data-screen-recording-used='false']"),
    ).toBeTruthy();
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
