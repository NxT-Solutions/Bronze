export const AUTOMATIC_UPLOAD = false;
export const HUMAN_GATES = ["3.9", "3.10", "5.5", "9.3"] as const;

export type BundlePreview = {
  eventCount: number;
  text: string;
};

export function previewBundle(eventCount: number): BundlePreview {
  return {
    eventCount,
    text: `events=${eventCount} limits=${HUMAN_GATES.join(",")}`,
  };
}

export function exportBundle(
  preview: BundlePreview,
  acknowledged: boolean,
): BundlePreview {
  if (!acknowledged) {
    throw new Error("PreviewRequired");
  }
  if (/secret|password|token|hunter2/i.test(preview.text)) {
    throw new Error("SecretDetected");
  }
  return preview;
}
