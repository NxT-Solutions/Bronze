export type WindowKind = "quick" | "library";
export type WindowCommand =
  | "import"
  | "export"
  | "backup"
  | "paginate"
  | "archive";

export function windowAllows(
  window: WindowKind,
  command: WindowCommand,
): boolean {
  if (window === "quick") {
    return false;
  }
  return (
    command === "paginate" ||
    command === "archive" ||
    command === "import" ||
    command === "export" ||
    command === "backup"
  );
}
