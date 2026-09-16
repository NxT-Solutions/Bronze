import { useId, useState } from "react";
import { Label, TextArea, TextField } from "react-aria-components";
import { Button } from "@/components/button";
import { cn } from "@/lib/utils";

export type ComposerDraft = {
  body: string;
  contentLanguage: string;
};

export function composerShouldAdd(input: {
  key: string;
  metaKey: boolean;
  isComposing: boolean;
}): boolean {
  return input.key === "Enter" && input.metaKey && !input.isComposing;
}

export function Composer({
  label,
  submitLabel,
  errorLabel,
  onAdd,
  className,
}: {
  label: string;
  submitLabel: string;
  errorLabel: string;
  onAdd: (draft: ComposerDraft) => Promise<void> | void;
  className?: string;
}) {
  const errorId = useId();
  const [body, setBody] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function submit() {
    try {
      await onAdd({ body, contentLanguage: "und" });
      setError(null);
      setBody("");
    } catch {
      setError(errorLabel);
    }
  }

  return (
    <form
      className={cn("grid gap-2", className)}
      data-slot="composer"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <TextField className="grid gap-1.5" value={body} onChange={setBody}>
        <Label className="text-sm font-medium text-foreground">{label}</Label>
        <TextArea
          id="composer-body"
          aria-describedby={error ? errorId : undefined}
          className="min-h-24 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm"
          onKeyDown={(event) => {
            if (
              composerShouldAdd({
                key: event.key,
                metaKey: event.metaKey,
                isComposing: event.nativeEvent.isComposing,
              })
            ) {
              event.preventDefault();
              void submit();
            }
          }}
        />
      </TextField>
      <Button type="submit">{submitLabel}</Button>
      {error ? (
        <p id={errorId} role="alert">
          {error}
        </p>
      ) : null}
    </form>
  );
}
