import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it, vi } from "vitest";
import { ItemList } from "@/components/item-list";

const labels = {
  moveUp: "Move up",
  moveDown: "Move down",
  complete: "Complete",
  skip: "Skip",
  trash: "Trash",
  edit: "Edit",
};

describe("ItemList (QUE-002, A11Y-002)", () => {
  it("uses list/article/buttons and keyboard operates actions", async () => {
    const user = userEvent.setup();
    const onAction = vi.fn();
    const { container } = render(
      <ItemList
        items={[{ id: "1", body: "park" }]}
        labels={labels}
        onAction={onAction}
      />,
    );
    expect(container.querySelector("ul")).toBeTruthy();
    expect(container.querySelector("article")).toBeTruthy();
    expect(container.querySelectorAll('div[role="button"]').length).toBe(0);
    for (const name of Object.values(labels)) {
      const button = screen.getByRole("button", { name });
      expect(button.textContent).toBe(name);
    }
    await user.tab();
    expect(screen.getByRole("button", { name: "Move up" })).toHaveFocus();
    await user.keyboard("{Enter}");
    expect(onAction).toHaveBeenCalledWith("1", "moveUp");
  });

  it("axe passes on the queue list", async () => {
    const { container } = render(
      <ItemList
        items={[{ id: "1", body: "park" }]}
        labels={labels}
        onAction={() => undefined}
      />,
    );
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
