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

test("four surfaces share zinc chrome and kill native appearance", () => {
  assert.match(chrome, /--text-micro:\s*0\.6875rem/);
  assert.match(chrome, /--text-caption:\s*0\.8125rem/);
  assert.match(chrome, /--text-body:\s*0\.9375rem/);
  assert.match(chrome, /--text-title:\s*1\.25rem/);
  assert.match(chrome, /--text-display:\s*1\.75rem/);
  assert.match(chrome, /--control-h:\s*2rem/);
  assert.match(chrome, /\[data-slot="source"\]/);
  assert.match(chrome, /\[data-slot="source"\]\[hidden\]/);
  assert.match(chrome, /article:not\(\.is-expanded\)\s*\[data-slot="body"\]/);
  assert.match(chrome, /\[data-slot="body"\][\s\S]*white-space:\s*pre-wrap/);
  assert.match(chrome, /\[data-slot="body"\][\s\S]*tab-size:\s*4/);
  assert.match(
    chrome,
    /article:not\(\.is-expanded\)\s*\[data-slot="body"\][\s\S]*max-height:\s*calc\(1\.45em \* 3\)/,
  );
  assert.doesNotMatch(chrome, /-webkit-line-clamp/);
  assert.doesNotMatch(chrome, /-webkit-box-orient/);
  assert.match(
    chrome,
    /\[data-slot="source-icon"\][\s\S]*object-fit:\s*contain/,
  );
  assert.match(
    chrome,
    /\[data-slot="source-icon"\][\s\S]*background:\s*transparent/,
  );
  assert.match(chrome, /\.empty-state\[hidden\]/);
  assert.match(chrome, /#composer-error\[hidden\]/);
  assert.match(chrome, /#capture-status\[hidden\]/);
  assert.match(chrome, /#action-status\[hidden\]/);
  assert.match(chrome, /\.action-tip\[hidden\]/);
  assert.match(chrome, /\.action-tip[\s\S]*position:\s*absolute/);
  assert.match(chrome, /\.action-tip\[data-tone="failed"\]/);
  assert.match(chrome, /\[data-capture-message\]\[hidden\]/);
  assert.match(chrome, /cursor:\s*pointer/);
  assert.match(chrome, /\[aria-busy="true"\]/);
  assert.match(
    chrome,
    /button\.btn-primary:hover:not\(:disabled\)[\s\S]*--primary-foreground/,
  );
  assert.match(
    chrome,
    /button\.btn-primary:disabled[\s\S]*--primary-foreground/,
  );
  assert.match(
    chrome,
    /button\.btn-primary\[aria-busy="true"\][\s\S]*--primary-foreground/,
  );
  assert.match(chrome, /\.row-actions menu[\s\S]*position:\s*absolute/);
  assert.match(chrome, /\.row-actions menu[\s\S]*flex-direction:\s*column/);
  assert.match(chrome, /summary::-webkit-details-marker/);
  assert.match(chrome, /article \[data-slot="body"\] p/);
  assert.match(chrome, /article \[data-slot="body"\] ul/);
  assert.match(chrome, /appearance:\s*none/);
  assert.match(chrome, /180ms/);
  assert.match(chrome, /cubic-bezier\(0\.22, 1, 0\.36, 1\)/);
  assert.match(chrome, /@keyframes bronze-enter/);
  assert.match(chrome, /@keyframes bronze-pop/);
  assert.match(chrome, /SF Pro Display/);
  assert.match(chrome, /optimizeLegibility/);
  assert.match(chrome, /article \{[\s\S]*box-shadow:\s*none/);
  assert.match(
    chrome,
    /article \[data-slot="title"\][\s\S]*font-weight:\s*590/,
  );
  assert.match(
    chrome,
    /article \[data-slot="body"\][\s\S]*font-size:\s*var\(--text-caption\)/,
  );
  assert.match(chrome, /\.action-tip[\s\S]*background:\s*var\(--card\)/);
  assert.match(chrome, /\.row-actions summary[\s\S]*background:\s*transparent/);
  assert.match(chrome, /\.row-actions menu button[\s\S]*border:\s*none/);
  assert.match(chrome, /\.field-row input[\s\S]*width:\s*auto/);
  assert.match(
    chrome,
    /\.shortcut-list li[\s\S]*justify-content:\s*space-between/,
  );
  assert.match(chrome, /article\.prose/);
  assert.match(chrome, /word-spacing:\s*0\.02em/);
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
