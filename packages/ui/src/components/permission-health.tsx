import { Button } from "@/components/button";
import {
  manualComposerAvailable,
  type PermissionCapability,
  type PermissionHealthRow,
  permissionHealthRows,
  SCREEN_RECORDING_USED,
} from "@/lib/permission-health";

export const PERMISSION_KEYS = {
  title: "settings.permission.title",
  retest: "settings.permission.retest",
  notUsed: "settings.permission.screenRecording.notUsed",
  composer: "settings.permission.composer.available",
  openSystemSettings: "settings.permission.openSystemSettings",
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
  revealedSettings = {},
  onRetest,
  onOpenSystemSettings,
}: {
  labels: Record<keyof typeof PERMISSION_KEYS, string>;
  rows?: PermissionHealthRow[];
  why: Record<string, string>;
  alternative: Record<string, string>;
  revealedSettings?: Partial<Record<PermissionCapability, boolean>>;
  onRetest?: (capability: PermissionCapability) => void;
  onOpenSystemSettings?: (capability: PermissionCapability) => void;
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
              <>
                <Button
                  type="button"
                  data-permission-retest={row.capability}
                  onPress={() => onRetest?.(row.capability)}
                >
                  {labels.retest}
                </Button>
                {(row.capability === "inputMonitoring" ||
                  row.capability === "accessibility") &&
                revealedSettings[row.capability] ? (
                  <Button
                    type="button"
                    data-permission-open-settings={row.capability}
                    onPress={() => onOpenSystemSettings?.(row.capability)}
                  >
                    {labels.openSystemSettings}
                  </Button>
                ) : null}
              </>
            )}
          </li>
        ))}
      </ul>
      {manualComposerAvailable() ? <p>{labels.composer}</p> : null}
    </section>
  );
}
