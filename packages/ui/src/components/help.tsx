import { Button } from "@/components/button";
import { AUTOMATIC_UPLOAD, HUMAN_GATES } from "@/lib/support";

export const HELP_KEYS = {
  title: "help.title",
  about: "help.about",
  preview: "help.diagnostics.preview",
  exportBundle: "help.diagnostics.export",
  humanGates: "help.limitations.humanGates",
  noUpload: "help.upload.none",
} as const;

export function Help({
  labels,
  preview,
  onExport,
}: {
  labels: Record<keyof typeof HELP_KEYS, string>;
  preview: string;
  onExport: () => void;
}) {
  return (
    <main
      data-slot="help"
      data-automatic-upload={AUTOMATIC_UPLOAD ? "true" : "false"}
    >
      <h1>{labels.title}</h1>
      <article>
        <h2>{labels.about}</h2>
        <p>{labels.noUpload}</p>
        <p>{labels.humanGates}</p>
        <ul>
          {HUMAN_GATES.map((gate) => (
            <li key={gate}>{gate}</li>
          ))}
        </ul>
      </article>
      <section aria-labelledby="diagnostics-preview-title">
        <h2 id="diagnostics-preview-title">{labels.preview}</h2>
        <pre data-diagnostics-preview>{preview}</pre>
        <Button type="button" onPress={onExport}>
          {labels.exportBundle}
        </Button>
      </section>
    </main>
  );
}
