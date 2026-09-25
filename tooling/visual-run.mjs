import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  PLAYWRIGHT_IMAGE,
  PLAYWRIGHT_VERSION,
} from "../apps/desktop/visual/pin.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const extra = process.argv.slice(2).filter((arg) => arg !== "--");

function shellQuote(value) {
  return `'${String(value).replaceAll("'", `'\\''`)}'`;
}

if (process.env.VISUAL_CONTAINER === "1") {
  const result = spawnSync(
    "npx",
    ["playwright", "test", "-c", "visual/playwright.config.mjs", ...extra],
    { cwd: join(root, "apps/desktop"), stdio: "inherit", env: process.env },
  );
  process.exit(result.status ?? 1);
}

const docker = spawnSync("docker", ["version"], { stdio: "ignore" });
if (docker.status !== 0) {
  console.error(
    "pnpm visual needs Docker. CI compares baselines from the pinned Playwright Linux image.",
  );
  process.exit(1);
}

const forwarded = extra.map(shellQuote).join(" ");
// Copy into the container filesystem. A read-write bind of the repo would
// follow pnpm's store symlinks and overwrite host binaries.
const inner = `
set -euo pipefail
mkdir -p /tmp/work
tar -C /src --warning=no-file-changed -cf - \\
  --exclude node_modules \\
  --exclude .git \\
  --exclude target \\
  --exclude apps/desktop/visual/test-results \\
  . | tar -C /tmp/work -xf -
cd /tmp/work/apps/desktop
npm install --no-save --ignore-scripts --no-audit --no-fund --no-package-lock @playwright/test@${PLAYWRIGHT_VERSION}
npx playwright install chromium
set +e
npx playwright test -c visual/playwright.config.mjs ${forwarded}
status=$?
mkdir -p /visual-out/chrome.visual.mjs-snapshots
rm -rf /visual-out/test-results
cp -a visual/test-results /visual-out/test-results 2>/dev/null || true
cp -a visual/chrome.visual.mjs-snapshots/. /visual-out/chrome.visual.mjs-snapshots/
chmod -R a+rX /visual-out/test-results /visual-out/chrome.visual.mjs-snapshots || true
exit $status
`;

// amd64 emulation on Apple Silicon cannot start this image's Chromium.
// The platform follows the host CPU. Ubuntu CI is amd64 and compares -x64 baselines.
const args = [
  "run",
  "--rm",
  "--platform",
  process.env.VISUAL_PLATFORM ??
    (process.arch === "arm64" ? "linux/arm64" : "linux/amd64"),
  "--ipc=host",
  "--shm-size=1gb",
  "-e",
  "CI=1",
  "-v",
  `${root}:/src:ro`,
  "-v",
  `${join(root, "apps/desktop/visual")}:/visual-out`,
  PLAYWRIGHT_IMAGE,
  "bash",
  "-lc",
  inner,
];

const run = spawnSync("docker", args, { stdio: "inherit" });
process.exit(run.status ?? 1);
