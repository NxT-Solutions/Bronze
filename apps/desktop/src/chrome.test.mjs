import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const pages = ["index.html", "library.html", "settings.html", "help.html"].map(
  (name) => ({
    name,
    html: readFileSync(join(root, name), "utf8"),
  }),
);

test("four surfaces share parchment chrome and kill native appearance", () => {
  assert.match(chrome, /--text-micro:\s*0\.6875rem/);
  assert.match(chrome, /--text-caption:\s*0\.8125rem/);
  assert.match(chrome, /--text-body:\s*0\.9375rem/);
  assert.match(chrome, /--text-title:\s*1\.25rem/);
  assert.match(chrome, /--text-display:\s*1\.75rem/);
  assert.match(chrome, /--control-h:\s*2rem/);
  assert.match(chrome, /appearance:\s*none/);
  assert.match(chrome, /160ms ease/);
  assert.doesNotMatch(
    chrome.toLowerCase(),
    /copper|cooper|@import|fonts\.google/,
  );
  for (const page of pages) {
    assert.match(page.html, /href="\.\/chrome\.css"/, page.name);
    assert.doesNotMatch(page.html, /#111|#eee/, page.name);
    assert.doesNotMatch(page.html.toLowerCase(), /copper|cooper/);
  }
  assert.match(pages[2].html, /settings\.permission\.status\.denied/);
  assert.match(pages[2].html, /data-status="notUsed"/);
});
