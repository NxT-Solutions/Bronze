import { Button } from "@/components/button";
import { windowAllows } from "@/lib/window-allows";

export type LibraryViewState = "empty" | "loading" | "ready" | "readOnly";

export const LIBRARY_KEYS = {
  title: "library.title",
  archive: "library.archive",
  paginate: "library.paginate",
  empty: "library.state.empty",
  loading: "library.state.loading",
  readOnly: "library.state.readOnly",
  backup: "library.backup.now",
  restore: "library.restore.preview",
  exportQueue: "library.export",
  importQueue: "library.import",
  secretWarning: "export.preview.secretBodies",
} as const;

export function Library({
  state,
  labels,
  onArchive,
  onPaginate,
}: {
  state: LibraryViewState;
  labels: Record<keyof typeof LIBRARY_KEYS, string>;
  onArchive: () => void;
  onPaginate: () => void;
}) {
  return (
    <main data-slot="library" data-library-state={state}>
      <h1>{labels.title}</h1>
      {state === "empty" ? <p role="status">{labels.empty}</p> : null}
      {state === "loading" ? <p role="status">{labels.loading}</p> : null}
      {state === "readOnly" ? <p role="status">{labels.readOnly}</p> : null}
      {windowAllows("library", "archive") && state !== "loading" ? (
        <Button
          type="button"
          isDisabled={state === "readOnly"}
          onPress={onArchive}
        >
          {labels.archive}
        </Button>
      ) : null}
      {windowAllows("library", "paginate") && state === "ready" ? (
        <Button type="button" onPress={onPaginate}>
          {labels.paginate}
        </Button>
      ) : null}
      {windowAllows("library", "backup") && state === "ready" ? (
        <Button type="button">{labels.backup}</Button>
      ) : null}
      {windowAllows("library", "export") && state === "ready" ? (
        <Button type="button">{labels.exportQueue}</Button>
      ) : null}
      {windowAllows("library", "import") && state === "ready" ? (
        <Button type="button">{labels.importQueue}</Button>
      ) : null}
      {state === "ready" ? <p data-restore-preview>{labels.restore}</p> : null}
      {state === "ready" ? (
        <p role="status" data-export-secret-warning>
          {labels.secretWarning}
        </p>
      ) : null}
    </main>
  );
}
