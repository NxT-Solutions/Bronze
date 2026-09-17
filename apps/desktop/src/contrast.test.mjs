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

test("quick panel uses zinc surfaces not competitor dress", () => {
  assert.match(html, /chrome\.css/);
  assert.match(chrome, /--background:\s*#fafafa/i);
  assert.match(chrome, /--foreground:\s*#18181b/i);
  assert.match(chrome, /--muted-foreground:\s*#52525b/i);
  assert.match(chrome, /--border:\s*#8a8a90/i);
  assert.match(chrome, /--stroke:\s*#8a8a90/i);
  assert.match(chrome, /--success:\s*#245a40/i);
  assert.match(chrome, /--warning:\s*#7a450c/i);
  assert.match(chrome, /--destructive:\s*#9b1c14/i);
  assert.doesNotMatch(chrome, /#f7f4ef|#8c6239|#1c1917/i);
  assert.doesNotMatch(css, /#f7f4ef|#8c6239|#1c1917/i);
  assert.doesNotMatch(html.toLowerCase(), /copper|cooper/);
  assert.doesNotMatch(chrome.toLowerCase(), /copper|cooper/);
  assert.doesNotMatch(css.toLowerCase(), /copper|cooper/);
  assert.match(css, /from DESIGN\.md/);
  assert.match(chrome, /from DESIGN\.md/);
});
