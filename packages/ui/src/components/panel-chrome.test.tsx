import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ItemList } from "@/components/item-list";
import { PanelChrome } from "@/components/panel-chrome";

const labels = {
  moveUp: "Move up",
  moveDown: "Move down",
  complete: "Complete",
  skip: "Skip",
  trash: "Trash",
  edit: "Edit",
};

describe("PanelChrome RTL smoke (I18N-002/003)", () => {
  it("sets chrome dir from locale and keeps a physical edge", () => {
    const { container } = render(
      <PanelChrome locale="ar-XB" edge="left" title="Bronze">
        <ItemList
          items={[{ id: "1", body: "مرحبا", contentLanguage: "ar" }]}
          labels={labels}
          onAction={() => undefined}
        />
      </PanelChrome>,
    );
    const chrome = container.querySelector('[data-slot="quick-panel"]');
    expect(chrome?.getAttribute("dir")).toBe("rtl");
    expect(chrome?.getAttribute("lang")).toBe("ar-XB");
    expect(chrome?.getAttribute("data-physical-edge")).toBe("left");
    const card = container.querySelector("article");
    expect(card?.getAttribute("lang")).toBe("ar");
    expect(card?.getAttribute("dir")).toBe("auto");
    expect(screen.getByRole("heading", { name: "Bronze" })).toBeTruthy();
  });
});
