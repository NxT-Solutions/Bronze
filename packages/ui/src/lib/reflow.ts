export const REFLOW_WIDTH_CSS_PX = 320;
export const TEXT_RESIZE_PERCENT = 200;
export const TWO_AXIS_SCROLL_ALLOWED = false;

export function twoAxisScroll(
  content: { width: number; height: number },
  viewport: { width: number; height: number },
): boolean {
  return content.width > viewport.width && content.height > viewport.height;
}
