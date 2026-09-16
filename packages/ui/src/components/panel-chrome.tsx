import type { ReactNode } from "react";
import { chromeDir } from "@/lib/catalog";

export type PhysicalEdge = "left" | "right" | "top";

export function PanelChrome({
  locale,
  edge,
  title,
  children,
}: {
  locale: string;
  edge: PhysicalEdge;
  title: string;
  children?: ReactNode;
}) {
  return (
    <main
      data-slot="quick-panel"
      data-physical-edge={edge}
      lang={locale}
      dir={chromeDir(locale)}
    >
      <header>
        <span aria-hidden="true" />
        <h1 data-i18n="panel.quick.title">{title}</h1>
      </header>
      {children}
    </main>
  );
}
