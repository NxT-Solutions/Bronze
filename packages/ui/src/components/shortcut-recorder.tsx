import { type KeyboardEvent, useState } from "react";
import { Button } from "@/components/button";
import {
  type CaptureAlternatives,
  recorderSwallows,
  registerShortcut,
  type ShortcutBinding,
  seedShortcutRegistry,
  standardChordAction,
} from "@/lib/shortcut-registry";

export const SHORTCUT_KEYS = {
  title: "settings.shortcuts.title",
  record: "settings.shortcuts.record",
  skipTest: "settings.shortcuts.skipTest",
  alternatives: "settings.shortcuts.alternatives",
  live: "settings.shortcuts.live",
} as const;

export function ShortcutRecorder({
  labels,
  nativeOk = true,
  alternatives = { chord: true, menu: true, manual: true },
}: {
  labels: Record<keyof typeof SHORTCUT_KEYS, string>;
  nativeOk?: boolean;
  alternatives?: CaptureAlternatives;
}) {
  const [bindings, setBindings] = useState(seedShortcutRegistry);
  const [live, setLive] = useState("");
  const standard = bindings.find(
    (binding) => binding.action === standardChordAction(),
  );

  function onKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (
      !recorderSwallows({
        isComposing: event.nativeEvent.isComposing,
        isVoiceOverReserved: event.code === "F5" && event.ctrlKey,
        isRepeat: event.repeat,
      })
    ) {
      return;
    }
    event.preventDefault();
    const chord = [event.metaKey ? "Meta" : "", event.key]
      .filter(Boolean)
      .join("+");
    setLive(chord);
    setBindings((current) =>
      registerShortcut(
        current,
        {
          action: "capture.selection",
          chord,
          enabled: true,
        },
        nativeOk,
        alternatives,
      ),
    );
  }

  return (
    <section data-slot="shortcut-recorder">
      <h2>{labels.title}</h2>
      <p role="status" data-shortcut-live>
        {live || labels.live}
      </p>
      <label htmlFor="shortcut-record">{labels.record}</label>
      <input
        id="shortcut-record"
        type="text"
        onKeyDown={onKeyDown}
        aria-describedby="shortcut-live-help"
      />
      <p id="shortcut-live-help">{labels.live}</p>
      <Button type="button" onPress={() => setLive("skipped")}>
        {labels.skipTest}
      </Button>
      <p data-standard-chord={standard?.action}>{standard?.chord}</p>
      <ul data-shortcut-registry>
        {bindings.map((binding: ShortcutBinding) => (
          <li key={binding.action} data-action={binding.action}>
            {binding.action}
          </li>
        ))}
      </ul>
      <p
        data-alternatives-ok={String(
          alternatives.chord || alternatives.menu || alternatives.manual,
        )}
      >
        {labels.alternatives}
      </p>
    </section>
  );
}
