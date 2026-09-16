import { Button } from "@/components/button";

export const COPY_KEYS = {
  profile: "copy.profile.label",
  copy: "copy.action.copy",
  preview: "copy.preview.label",
} as const;

export type OutputProfileOption = {
  id: string;
  name: string;
};

export function CopyToolbar({
  profiles,
  selectedId,
  preview,
  labels,
  onSelect,
  onCopy,
}: {
  profiles: OutputProfileOption[];
  selectedId: string;
  preview: string;
  labels: Record<keyof typeof COPY_KEYS, string>;
  onSelect: (id: string) => void;
  onCopy: () => void;
}) {
  return (
    <div data-slot="copy-toolbar">
      <label htmlFor="output-profile">{labels.profile}</label>
      <select
        id="output-profile"
        value={selectedId}
        onChange={(event) => onSelect(event.target.value)}
      >
        {profiles.map((profile) => (
          <option key={profile.id} value={profile.id}>
            {profile.name}
          </option>
        ))}
      </select>
      <Button type="button" onPress={onCopy}>
        {labels.copy}
      </Button>
      <output>
        <strong>{labels.preview}</strong>
        <pre>{preview}</pre>
      </output>
    </div>
  );
}
