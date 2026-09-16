import { Button } from "@/components/button";
import {
  manualComposerAvailable,
  type PermissionHealthRow,
  permissionHealthRows,
  SCREEN_RECORDING_USED,
} from "@/lib/permission-health";

export const PERMISSION_KEYS = {
  title: "settings.permission.title",
  retest: "settings.permission.retest",
  notUsed: "settings.permission.screenRecording.notUsed",
  composer: "settings.permission.composer.available",
} as const;

export function PermissionHealth({
  labels,
  rows = permissionHealthRows({
    inputMonitoring: "denied",
    accessibility: "denied",
    selfTest: "unknown",
  }),
  why,
  alternative,
}: {
  labels: Record<keyof typeof PERMISSION_KEYS, string>;
  rows?: PermissionHealthRow[];
  why: Record<string, string>;
  alternative: Record<string, string>;
}) {
  return (
    <section data-slot="permission-health">
      <h2>{labels.title}</h2>
      <ul>
        {rows.map((row) => (
          <li
            key={row.capability}
            data-capability={row.capability}
            data-usage={row.usage}
          >
            <p>{why[row.capability]}</p>
            <p>{alternative[row.capability]}</p>
            {row.capability === "screenRecording" ? (
              <p data-screen-recording-used={String(SCREEN_RECORDING_USED)}>
                {labels.notUsed}
              </p>
            ) : (
              <Button type="button">{labels.retest}</Button>
            )}
          </li>
        ))}
      </ul>
      {manualComposerAvailable() ? <p>{labels.composer}</p> : null}
    </section>
  );
}
