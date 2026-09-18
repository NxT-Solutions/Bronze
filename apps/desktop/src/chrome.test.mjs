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
  assert.match(chrome, /\.page-shell \[hidden\][\s\S]*display:\s*none/);
  assert.match(chrome, /\.chrome-search/);
  assert.match(chrome, /\.chrome-search-icon/);
  assert.match(chrome, /\.chrome-search input::placeholder/);
  assert.match(chrome, /\.chrome-search:focus-within/);
  assert.match(chrome, /\.chrome-search-clear/);
  assert.match(chrome, /\.chrome-search-clear\[hidden\]/);
  assert.match(chrome, /\.app-picker-option-icon/);
  assert.match(chrome, /\.pill\[data-status="unavailable"\]/);
  assert.match(chrome, /\.app-picker-search/);
  assert.match(chrome, /\.field-help/);
  assert.match(chrome, /\.btn-icon/);
  assert.match(chrome, /#composer-error\[hidden\]/);
  assert.match(chrome, /#capture-status\[hidden\]/);
  assert.match(chrome, /#action-status\[hidden\]/);
  assert.match(chrome, /\.chrome-notice\[hidden\]/);
  assert.match(chrome, /\.chrome-notice[\s\S]*position:\s*fixed/);
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
  assert.match(chrome, /#queue[\s\S]*isolation:\s*isolate/);
  assert.match(chrome, /\.queue-item[\s\S]*z-index:\s*0/);
  assert.match(
    chrome,
    /\.queue-item:has\(details\[open\]\)[\s\S]*z-index:\s*3/,
  );
  assert.match(chrome, /\.row-actions details\[open\][\s\S]*z-index:\s*5/);
  assert.match(chrome, /\.row-actions menu[\s\S]*z-index:\s*6/);
  assert.match(
    chrome,
    /\.row-actions menu \[data-queue-action="trash"\][\s\S]*--destructive/,
  );
  assert.match(chrome, /\.segment a\[aria-current="page"\]/);
  assert.match(chrome, /\.help-launch[\s\S]*justify-content:\s*flex-end/);
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
  assert.match(chrome, /line-height:\s*1\.5/);
  assert.match(chrome, /\.skip-link/);
  assert.match(chrome, /max-width:\s*80ch/);
  assert.match(chrome, /#library-queue/);
  assert.match(chrome, /\.page-shell \.card[\s\S]*gap:\s*var\(--space-4\)/);
  assert.match(chrome, /\.library-dock/);
  assert.match(chrome, /\[data-standard-chord\]:empty/);
  assert.match(chrome, /--radius-control:\s*8px/);
  assert.match(
    chrome,
    /--radius-page-card:\s*calc\(var\(--radius-control\) \+ var\(--space-5\)\)/,
  );
  assert.match(
    chrome,
    /\.page-shell \.card[\s\S]*border-radius:\s*var\(--radius-page-card\)/,
  );
  assert.match(chrome, /\.pill[\s\S]*border-radius:\s*var\(--radius-pill\)/);
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
  assert.match(pages[0].html, /class="queue-item"/);
  assert.match(pages[1].html, /class="queue-item"/);
  assert.match(pages[2].html, /class="help-launch"/);
  assert.match(pages[0].html, /id="edit-sheet"/);
  assert.match(chrome, /\.sheet-card[\s\S]*background:\s*var\(--card\)/);
  assert.match(chrome, /button\.btn-ghost\[type="submit"\]/);
  assert.doesNotMatch(chrome, /backdrop-filter|filter:\s*blur/);
  assert.match(chrome, /fieldset \{[\s\S]*?border:\s*none/);
  assert.match(
    chrome,
    /fieldset \{[\s\S]*?box-shadow:\s*0 0 0 1px var\(--stroke\)/,
  );
  assert.match(chrome, /legend \{[\s\S]*?display:\s*contents/);
});
