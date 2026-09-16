import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it, vi } from "vitest";
import { QUE_007_COMPLETE, SearchField } from "@/components/search-field";

describe("SearchField (QUE-007, A11Y-002)", () => {
  it("announces count without the query and leaves QUE-007 incomplete", async () => {
    expect(QUE_007_COMPLETE).toBe(false);
    const user = userEvent.setup();
    const onQuery = vi.fn();
    const { container } = render(
      <SearchField
        label="Search"
        countLabel="1 item"
        value=""
        onQuery={onQuery}
      />,
    );
    await user.type(
      screen.getByRole("searchbox", { name: "Search" }),
      "secret",
    );
    expect(onQuery).toHaveBeenCalled();
    expect(screen.getByRole("status")).toHaveTextContent("1 item");
    expect(screen.getByRole("status").textContent).not.toContain("secret");
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
