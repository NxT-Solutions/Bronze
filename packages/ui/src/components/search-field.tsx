export const QUE_007_COMPLETE = false;

export function SearchField({
  label,
  countLabel,
  value,
  onQuery,
}: {
  label: string;
  countLabel: string;
  value: string;
  onQuery: (query: string) => void;
}) {
  return (
    <search data-slot="search-field">
      <label htmlFor="library-search">{label}</label>
      <input
        id="library-search"
        type="search"
        value={value}
        onChange={(event) => onQuery(event.target.value)}
        className="h-8 w-full rounded-[8px] border border-input bg-background px-3 text-[0.8125rem]"
      />
      <p role="status" data-search-count>
        {countLabel}
      </p>
    </search>
  );
}
