import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { describe, expect, it } from "vitest";
import { Button } from "@/components/button";
import { Dialog, DialogContent, DialogTrigger } from "@/components/dialog";

describe("Dialog (React Aria base)", () => {
  it("renders trigger button and opens with role dialog, name, and focus", async () => {
    const user = userEvent.setup();
    render(
      <DialogTrigger>
        <Button>Open</Button>
        <DialogContent>
          <Dialog aria-label="Confirm">Content</Dialog>
        </DialogContent>
      </DialogTrigger>,
    );
    await user.click(screen.getByRole("button", { name: "Open" }));
    const dlg = await screen.findByRole("dialog", { name: "Confirm" });
    expect(dlg).toBeInTheDocument();
    expect(dlg.contains(document.activeElement)).toBe(true);
    expect(dlg.className).not.toMatch(/outline-none/);
  });

  it("axe on closed trigger state", async () => {
    const { container } = render(
      <DialogTrigger>
        <Button>open</Button>
        <DialogContent>
          <Dialog>hi</Dialog>
        </DialogContent>
      </DialogTrigger>,
    );
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
  });

  it("axe on open dialog", async () => {
    const user = userEvent.setup();
    render(
      <DialogTrigger>
        <Button>Open</Button>
        <DialogContent>
          <Dialog aria-label="Confirm">Content</Dialog>
        </DialogContent>
      </DialogTrigger>,
    );
    await user.click(screen.getByRole("button", { name: "Open" }));
    const dlg = await screen.findByRole("dialog", { name: "Confirm" });
    expect((await axe.run(dlg)).violations).toEqual([]);
  });
});
