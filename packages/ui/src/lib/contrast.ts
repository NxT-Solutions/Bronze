/** WCAG 2 relative-luminance contrast (A11Y-003). Not a public AA claim. */

export const MIN_NORMAL_TEXT_CONTRAST = 4.5;
export const MIN_ENHANCED_TEXT_CONTRAST = 7;
export const MIN_NON_TEXT_CONTRAST = 3;
export const CONCEPT_PNG_IS_PIXEL_SPEC = false;

export type Rgb = { r: number; g: number; b: number };

export function parseHex(hex: string): Rgb {
  const raw = hex.replace("#", "");
  if (raw.length !== 6) {
    throw new Error("hex unavailable");
  }
  return {
    r: Number.parseInt(raw.slice(0, 2), 16),
    g: Number.parseInt(raw.slice(2, 4), 16),
    b: Number.parseInt(raw.slice(4, 6), 16),
  };
}

function channel(c: number): number {
  const s = c / 255;
  return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

export function relativeLuminance(rgb: Rgb): number {
  return (
    0.2126 * channel(rgb.r) + 0.7152 * channel(rgb.g) + 0.0722 * channel(rgb.b)
  );
}

export function contrastRatio(a: string, b: string): number {
  const la = relativeLuminance(parseHex(a));
  const lb = relativeLuminance(parseHex(b));
  const [hi, lo] = la > lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}

export function blockVars(
  css: string,
  selector: string,
): Record<string, string> {
  const start = css.indexOf(selector);
  if (start < 0) {
    throw new Error(`${selector} unavailable`);
  }
  const open = css.indexOf("{", start);
  const close = css.indexOf("}", open);
  const body = css.slice(open + 1, close);
  const out: Record<string, string> = {};
  for (const line of body.split(";")) {
    const match = line.match(/--([a-z-]+):\s*(#[0-9a-fA-F]{6})/);
    if (match) {
      out[match[1]] = match[2];
    }
  }
  return out;
}

export const TEXT_PAIRS = [
  ["foreground", "background"],
  ["card-foreground", "card"],
  ["muted-foreground", "muted"],
  ["primary-foreground", "primary"],
  ["accent-foreground", "accent"],
  ["secondary-foreground", "secondary"],
] as const;

export const PILL_PAIRS = [
  ["success-foreground", "success"],
  ["warning-foreground", "warning"],
  ["destructive-foreground", "destructive"],
] as const;

export const NON_TEXT_PAIRS = [
  ["border", "background"],
  ["border", "card"],
  ["input", "background"],
] as const;
