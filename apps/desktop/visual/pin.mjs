// Image tag and @playwright/test stay on the same release. Baselines are
// pixels from this image; a one-sided bump fails every shot.
export const PLAYWRIGHT_VERSION = "1.63.0";

export const PLAYWRIGHT_IMAGE = `mcr.microsoft.com/playwright:v${PLAYWRIGHT_VERSION}-jammy`;
