import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it, vi } from "vitest";
import { Library } from "@/components/library";

const labels = {
  title: "Library",
  archive: "Archive",
  paginate: "Next page",
  empty: "No items",
  loading: "Loading",
  readOnly: "Read only",
};

describe("Library (QUE-008, WIN-005)", () => {
  it("paginates and archives from buttons in ready state", async () => {
    const user = userEvent.setup();
    const onArchive = vi.fn();
    const onPaginate = vi.fn();
    render(
      <Library
        state="ready"
        labels={labels}
        onArchive={onArchive}
        onPaginate={onPaginate}
      />,
    );
    await user.click(screen.getByRole("button", { name: "Archive" }));
    await user.click(screen.getByRole("button", { name: "Next page" }));
    expect(onArchive).toHaveBeenCalledTimes(1);
    expect(onPaginate).toHaveBeenCalledTimes(1);
  });

  it("exposes empty loading and read-only states", async () => {
    const { rerender, container } = render(
      <Library
        state="empty"
        labels={labels}
        onArchive={() => undefined}
        onPaginate={() => undefined}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("No items");
    expect(
      container.querySelector("[data-library-state='empty']"),
    ).toBeTruthy();
    rerender(
      <Library
        state="loading"
        labels={labels}
        onArchive={() => undefined}
        onPaginate={() => undefined}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading");
    rerender(
      <Library
        state="readOnly"
        labels={labels}
        onArchive={() => undefined}
        onPaginate={() => undefined}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Read only");
    expect(screen.getByRole("button", { name: "Archive" })).toBeDisabled();
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
