import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it } from "vitest";
import { TextField } from "@/components/text-field";

describe("TextField (React Aria base)", () => {
  it("renders with role textbox and label association", () => {
    render(<TextField label="Title" />);
    const input = screen.getByRole("textbox", { name: "Title" });
    expect(input).toBeInTheDocument();
  });

  it("supports focus and keeps a focus-visible ring replacement", async () => {
    const user = userEvent.setup();
    render(<TextField label="Note" />);
    const tf = screen.getByRole("textbox", { name: "Note" });
    await user.tab();
    expect(tf).toHaveFocus();
    expect(tf.className).toMatch(/focus-visible:ring-/);
    expect(tf.className).not.toMatch(/outline-none/);
  });

  it("exposes error text as an alert, not color only", () => {
    render(<TextField label="Title" error="Required" />);
    expect(screen.getByRole("alert")).toHaveTextContent("Required");
    const input = screen.getByRole("textbox", { name: "Title" });
    expect(input).toHaveAttribute("aria-invalid", "true");
    expect(input).toHaveAccessibleDescription("Required");
  });

  it("axe passes on empty and focus states", async () => {
    const user = userEvent.setup();
    const { container } = render(<TextField label="Search" />);
    expect((await axe.run(container)).violations).toEqual([]);
    await user.tab();
    expect(screen.getByRole("textbox", { name: "Search" })).toHaveFocus();
    expect((await axe.run(container)).violations).toEqual([]);
  });

  it("axe passes on error state", async () => {
    const { container } = render(<TextField label="Title" error="Required" />);
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
