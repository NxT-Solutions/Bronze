import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it } from "vitest";
import { Button } from "@/components/button";

describe("Button (React Aria base)", () => {
  it("renders with role button and accessible name", () => {
    render(<Button>Save</Button>);
    const btn = screen.getByRole("button", { name: "Save" });
    expect(btn).toBeInTheDocument();
  });

  it("supports focus and keeps a focus-visible ring replacement", async () => {
    const user = userEvent.setup();
    render(<Button>Focus me</Button>);
    const btn = screen.getByRole("button", { name: "Focus me" });
    await user.tab();
    expect(btn).toHaveFocus();
    expect(btn.className).toMatch(/focus-visible:ring-/);
    expect(btn.className).not.toMatch(/outline-none/);
  });

  it("does not use clickable div", () => {
    const { container } = render(<Button>ok</Button>);
    const divs = container.querySelectorAll('div[role="button"], div[onclick]');
    expect(divs.length).toBe(0);
  });

  it("axe passes on default and focus states", async () => {
    const user = userEvent.setup();
    const { container } = render(<Button>Submit</Button>);
    expect((await axe.run(container)).violations).toEqual([]);
    await user.tab();
    expect(screen.getByRole("button", { name: "Submit" })).toHaveFocus();
    expect((await axe.run(container)).violations).toEqual([]);
  });
});
