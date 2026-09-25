import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it } from "vitest";
import { SettingsForm } from "@/components/settings-form";

const labels = {
  title: "Settings",
  search: "Search settings",
  general: "General",
  capture: "Capture",
  panel: "Panel",
  copy: "Copy",
  privacy: "Privacy",
  data: "Data",
  accessibility: "Accessibility",
  launchAtLogin: "Launch at login",
  backupSchedule: "Backup schedule",
  excludedBundleIds: "Excluded apps",
  daily: "Daily",
  weekly: "Weekly",
  resetField: "Reset field",
  resetGroup: "Reset group",
  resetAll: "Reset all",
  exportPreview: "Export preview",
  exportSensitive: "User-entered literals may be sensitive",
};

describe("SettingsForm (SET-001, WIN-005)", () => {
  it("searches groups, resets backup to daily, and flags export literals", async () => {
    const user = userEvent.setup();
    const { container } = render(
      <SettingsForm
        labels={labels}
        extraRaw={{
          permissionToken: "abc",
          logPath: "/Users/me/bronze.log",
        }}
      />,
    );
    expect(screen.getByRole("heading", { name: "Settings" })).toBeTruthy();
    expect(screen.queryByRole("option", { name: "off" })).toBeNull();
    const search = screen.getByRole("searchbox", { name: "Search settings" });
    await user.type(search, "backup");
    expect(screen.getByLabelText("Backup schedule")).toBeTruthy();
    expect(screen.queryByLabelText("Launch at login")).toBeNull();
    await user.selectOptions(
      screen.getByLabelText("Backup schedule"),
      "weekly",
    );
    expect(screen.getByLabelText("Backup schedule")).toHaveValue("weekly");
    await user.click(screen.getByRole("button", { name: "Reset field" }));
    expect(screen.getByLabelText("Backup schedule")).toHaveValue("daily");
    await user.clear(search);
    await user.type(screen.getByLabelText("Excluded apps"), "com.bank.app");
    expect(
      screen.getByText("User-entered literals may be sensitive"),
    ).toBeTruthy();
    expect(screen.getByText("privacy.excludedBundleIds")).toBeTruthy();
    expect(screen.getByText("permissionToken")).toBeTruthy();
    const json =
      container.querySelector("[data-export-json]")?.textContent ?? "";
    expect(json).toContain("daily");
    expect(json).not.toContain("abc");
    expect(json).not.toContain("/Users/me");
    expect((await axe.run(container)).violations).toEqual([]);
  });

  it("hides every settings group when the search matches nothing", async () => {
    const user = userEvent.setup();
    const { container } = render(<SettingsForm labels={labels} />);
    expect(
      container.querySelectorAll("[data-settings-group]").length,
    ).toBeGreaterThan(0);
    await user.type(
      screen.getByRole("searchbox", { name: "Search settings" }),
      "zzzz-no-such-setting",
    );
    expect(container.querySelector("[data-settings-group]")).toBeNull();
    expect(screen.queryByLabelText("Backup schedule")).toBeNull();
    expect(screen.queryByLabelText("Launch at login")).toBeNull();
    expect(screen.getByRole("heading", { name: "Settings" })).toBeTruthy();
  });
});
