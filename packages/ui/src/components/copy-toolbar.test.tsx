import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it, vi } from "vitest";
import { CopyToolbar } from "@/components/copy-toolbar";

describe("CopyToolbar (QUE-004, QUE-005)", () => {
  it("copies through a named profile button and shows preview", async () => {
    const user = userEvent.setup();
    const onCopy = vi.fn();
    const { container } = render(
      <CopyToolbar
        profiles={[{ id: "plain", name: "Plain" }]}
        selectedId="plain"
        preview="park"
        labels={{
          profile: "Output profile",
          copy: "Copy",
          preview: "Preview",
          overflow: "More actions",
        }}
        onSelect={() => undefined}
        onCopy={onCopy}
      />,
    );
    expect(screen.getByLabelText("Output profile")).toHaveValue("plain");
    expect(screen.getByRole("status")).toHaveTextContent("park");
    expect(screen.getByText("More actions")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "Copy" }));
    expect(onCopy).toHaveBeenCalledTimes(1);
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
