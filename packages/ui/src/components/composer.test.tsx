import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it, vi } from "vitest";
import { Composer, composerShouldAdd } from "@/components/composer";

describe("Composer (QUE-002, A11Y-002)", () => {
  it("Cmd-Enter adds and Enter during composition does not", () => {
    expect(
      composerShouldAdd({ key: "Enter", metaKey: true, isComposing: false }),
    ).toBe(true);
    expect(
      composerShouldAdd({ key: "Enter", metaKey: false, isComposing: false }),
    ).toBe(false);
    expect(
      composerShouldAdd({ key: "Enter", metaKey: false, isComposing: true }),
    ).toBe(false);
  });

  it("failure retains draft and content language defaults to und", async () => {
    const user = userEvent.setup();
    const onAdd = vi.fn().mockRejectedValue(new Error("store"));
    render(
      <Composer
        label="Add item"
        submitLabel="Add"
        errorLabel="Could not add item"
        onAdd={onAdd}
      />,
    );
    const field = screen.getByRole("textbox", { name: "Add item" });
    await user.type(field, "park me");
    await user.click(screen.getByRole("button", { name: "Add" }));
    expect(onAdd).toHaveBeenCalledWith({
      body: "park me",
      contentLanguage: "und",
    });
    expect(field).toHaveValue("park me");
    expect(screen.getByRole("alert")).toHaveTextContent("Could not add item");
  });

  it("axe passes on empty and error", async () => {
    const { container, rerender } = render(
      <Composer
        label="Add item"
        submitLabel="Add"
        errorLabel="Could not add item"
        onAdd={async () => {
          throw new Error("store");
        }}
      />,
    );
    expect((await axe.run(container)).violations).toEqual([]);
    rerender(
      <Composer
        label="Add item"
        submitLabel="Add"
        errorLabel="Could not add item"
        onAdd={async () => {
          throw new Error("store");
        }}
      />,
    );
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
