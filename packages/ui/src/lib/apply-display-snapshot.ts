export type DisplayPreferenceSnapshot = {
  reduceMotion: boolean;
  reduceTransparency: boolean;
  increaseContrast: boolean;
  differentiateWithoutColor: boolean;
};

const ATTRS = {
  reduceMotion: "data-reduce-motion",
  reduceTransparency: "data-reduce-transparency",
  increaseContrast: "data-increase-contrast",
  differentiateWithoutColor: "data-differentiate-without-color",
} as const;

export function applyDisplaySnapshot(
  root: HTMLElement,
  snapshot: DisplayPreferenceSnapshot,
): void {
  root.toggleAttribute(ATTRS.reduceMotion, snapshot.reduceMotion);
  root.toggleAttribute(ATTRS.reduceTransparency, snapshot.reduceTransparency);
  root.toggleAttribute(ATTRS.increaseContrast, snapshot.increaseContrast);
  root.toggleAttribute(
    ATTRS.differentiateWithoutColor,
    snapshot.differentiateWithoutColor,
  );
}

export function readDisplaySnapshot(
  root: HTMLElement,
): DisplayPreferenceSnapshot {
  return {
    reduceMotion: root.hasAttribute(ATTRS.reduceMotion),
    reduceTransparency: root.hasAttribute(ATTRS.reduceTransparency),
    increaseContrast: root.hasAttribute(ATTRS.increaseContrast),
    differentiateWithoutColor: root.hasAttribute(
      ATTRS.differentiateWithoutColor,
    ),
  };
}
