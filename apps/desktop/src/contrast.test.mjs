import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const css = readFileSync(
  join(root, "../../../packages/ui/src/styles/globals.css"),
  "utf8",
);

test("quick panel uses warm bronze surfaces not competitor dress", () => {
  assert.match(html, /chrome\.css/);
  assert.match(chrome, /background:\s*#f7f4ef/i);
  assert.match(chrome, /--foreground:\s*#1c1917/i);
  assert.doesNotMatch(html.toLowerCase(), /copper|cooper/);
  assert.doesNotMatch(chrome.toLowerCase(), /copper|cooper/);
  assert.doesNotMatch(css.toLowerCase(), /copper|cooper/);
  assert.match(css, /from DESIGN\.md/);
  assert.match(chrome, /from DESIGN\.md/);
});
