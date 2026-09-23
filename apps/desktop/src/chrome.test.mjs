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
  assert.match(
    chrome,
    /\.app-picker-option[\s\S]*justify-content:\s*flex-start/,
  );
  assert.match(chrome, /\.app-picker-list[\s\S]*align-items:\s*stretch/);
  assert.match(chrome, /\.pill\[data-status="unavailable"\]/);
  assert.match(chrome, /\.app-picker-search/);
  assert.match(chrome, /\.field-help/);
  assert.match(chrome, /\.setting-info/);
  assert.match(chrome, /\.setting-info-mark/);
  assert.match(chrome, /\.setting-info-panel/);
  assert.match(chrome, /\.setting-info-panel::before/);
  assert.match(chrome, /\.setting-info-panel[\s\S]*word-spacing:\s*0\.02em/);
  assert.match(chrome, /\.setting-info-panel strong[\s\S]*font-weight:\s*600/);
  assert.match(chrome, /\.field-help strong[\s\S]*font-weight:\s*600/);
  assert.match(chrome, /\.setting-info > summary[\s\S]*cursor:\s*help/);
  assert.match(pages[2].html, /class="setting-info"/);
  assert.equal((pages[2].html.match(/class="setting-info"/g) || []).length, 12);
  assert.match(pages[2].html, /settings\.shortcuts\.title\.infoRestore/);
  assert.match(pages[2].html, /settings\.field\.titleModel\.helpExtractive/);
  assert.match(pages[2].html, /settings\.field\.titleModel\.infoExtractive/);
  assert.match(pages[2].html, /settings\.field\.reduceMotion\.infoAlways/);
  assert.match(pages[2].html, /settings\.field\.reduceMotion\.infoPlay/);
  assert.match(pages[2].html, /<strong>Record<\/strong>/);
  assert.match(pages[2].html, /<strong>Restore<\/strong>/);
  assert.match(pages[2].html, /<strong>Stays on this Mac<\/strong>/);
  assert.match(pages[2].html, /<strong>Extractive<\/strong>/);
  assert.match(pages[2].html, /<strong>Follow this Mac<\/strong>/);
  assert.match(pages[2].html, /<strong>Always reduce<\/strong>/);
  assert.match(pages[2].html, /<strong>Play animations<\/strong>/);
  assert.match(chrome, /input\[type="checkbox"\]/);
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
  assert.match(chrome, /\.title-engine-spinner/);
  assert.match(chrome, /prefers-reduced-motion:\s*reduce/);
  assert.match(chrome, /html:not\(\[data-motion="full"\]\)/);
  assert.match(chrome, /html\[data-motion="reduce"\]/);
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
  const rowActionsRule = chrome.match(/\.row-actions\s*\{[^}]+\}/);
  assert.ok(rowActionsRule, "queue card action row rule");
  assert.match(rowActionsRule[0], /flex-wrap:\s*nowrap/);
  assert.match(rowActionsRule[0], /align-items:\s*center/);
  assert.match(rowActionsRule[0], /justify-content:\s*space-between/);
  assert.doesNotMatch(rowActionsRule[0], /justify-content:\s*flex-end/);
  assert.doesNotMatch(rowActionsRule[0], /justify-content:\s*flex-start/);
  assert.doesNotMatch(rowActionsRule[0], /margin-inline-start:\s*auto/);
  assert.doesNotMatch(rowActionsRule[0], /width:\s*fit-content/);
  assert.doesNotMatch(rowActionsRule[0], /\bright:\s/);
  assert.match(
    chrome,
    /\.row-actions > \.btn-primary\s*\{[\s\S]*?flex:\s*0 0 auto/,
  );
  const iconsRule = chrome.match(/\.row-action-icons\s*\{[^}]+\}/);
  assert.ok(iconsRule, "queue card icon row rule");
  assert.match(iconsRule[0], /flex-wrap:\s*nowrap/);
  assert.match(iconsRule[0], /flex:\s*0 1 auto/);
  assert.doesNotMatch(iconsRule[0], /width:\s*100%/);
  assert.doesNotMatch(iconsRule[0], /flex:\s*1 0 100%/);
  assert.match(
    pages[0].html,
    /data-queue-action="copy"[\s\S]*data-slot="action-icons"/,
  );
  assert.match(
    chrome,
    /\.row-action-icons \.icon-tip[\s\S]*position:\s*absolute/,
  );
  assert.match(chrome, /#queue[\s\S]*isolation:\s*isolate/);
  assert.match(chrome, /\.queue-item[\s\S]*z-index:\s*0/);
  assert.match(
    chrome,
    /\.queue-item:has\(\.row-action-icons \.btn-icon:hover:not\(:disabled\)\)[\s\S]*z-index:\s*3/,
  );
  assert.match(
    chrome,
    /\.row-action-icons \.btn-icon:hover:not\(:disabled\) \.icon-tip[\s\S]*pointer-events:\s*auto/,
  );
  assert.match(
    chrome,
    /\.row-action-icons \.btn-icon:focus-visible:not\(:disabled\) \.icon-tip[\s\S]*pointer-events:\s*auto/,
  );
  assert.match(
    chrome,
    /\.row-action-icons \.btn-icon:disabled \.icon-tip[\s\S]*visibility:\s*hidden/,
  );
  assert.match(
    chrome,
    /\.row-action-icons \.btn-icon:disabled[\s\S]*cursor:\s*default/,
  );
  assert.match(
    chrome,
    /\.row-action-icons \[data-queue-action="trash"\][\s\S]*--destructive/,
  );
  assert.match(chrome, /\.segment a\[aria-current="page"\]/);
  assert.match(chrome, /\.help-launch[\s\S]*justify-content:\s*flex-end/);
  assert.match(chrome, /summary::-webkit-details-marker/);
  assert.match(chrome, /article \[data-slot="body"\] p/);
  assert.match(chrome, /article \[data-slot="body"\] ul/);
  assert.match(
    chrome,
    /article \[data-slot="body"\][\s\S]*-webkit-user-select:\s*text/,
  );
  assert.match(
    chrome,
    /article \[data-slot="body"\][\s\S]*user-select:\s*text/,
  );
  assert.match(chrome, /\.composer-body[\s\S]*user-select:\s*text/);
  assert.match(chrome, /article\.prose[\s\S]*user-select:\s*text/);
  assert.match(
    chrome,
    /article \[data-slot="source-label"\][\s\S]*user-select:\s*text/,
  );
  assert.match(chrome, /\.page-shell \.card[\s\S]*user-select:\s*text/);
  assert.match(chrome, /\.page-shell fieldset[\s\S]*user-select:\s*text/);
  assert.match(chrome, /appearance:\s*none/);
  assert.match(chrome, /180ms/);
  assert.match(chrome, /cubic-bezier\(0\.22, 1, 0\.36, 1\)/);
  assert.match(chrome, /@keyframes bronze-enter/);
  assert.match(chrome, /@keyframes bronze-pop/);
  assert.match(chrome, /\.queue-item\.is-leaving-complete/);
  assert.match(chrome, /\.queue-item\.is-leaving-skip/);
  assert.match(chrome, /\.queue-item\.is-leaving-trash/);
  assert.match(
    chrome,
    /\.queue-item\.is-leaving-complete[\s\S]*grid-template-rows:\s*0fr/,
  );
  assert.match(
    chrome,
    /\.queue-item\.is-leaving-complete[\s\S]*translateY\(-8px\)/,
  );
  assert.match(chrome, /\.queue-item\.is-leaving-trash[\s\S]*scale\(0\.96\)/);
  assert.match(chrome, /SF Pro Display/);
  assert.match(chrome, /optimizeLegibility/);
  assert.match(chrome, /article \{[\s\S]*box-shadow:\s*none/);
  assert.match(
    chrome,
    /article \[data-slot="title"\][\s\S]*font-weight:\s*590/,
  );
  const titleRule = chrome.match(/article \[data-slot="title"\]\s*\{[^}]+\}/);
  assert.ok(titleRule, "queue card title rule");
  assert.match(titleRule[0], /text-overflow:\s*clip/);
  assert.match(titleRule[0], /white-space:\s*normal/);
  assert.doesNotMatch(titleRule[0], /text-overflow:\s*ellipsis/);
  assert.doesNotMatch(titleRule[0], /-webkit-line-clamp/);
  assert.match(
    chrome,
    /article \[data-slot="source"\][\s\S]*text-overflow:\s*ellipsis/,
  );
  assert.match(
    chrome,
    /article:not\(\.is-expanded\) \[data-slot="body"\][\s\S]*max-height/,
  );
  assert.match(
    chrome,
    /article \[data-slot="body"\][\s\S]*font-size:\s*var\(--text-caption\)/,
  );
  assert.match(chrome, /\.action-tip[\s\S]*background:\s*var\(--card\)/);
  assert.match(
    chrome,
    /\.row-action-icons \.icon-tip[\s\S]*background:\s*var\(--card\)/,
  );
  assert.match(chrome, /\.row-action-icons\[data-tips-dismissed\] \.icon-tip/);
  assert.match(pages[0].html, /data-slot="action-icons"/);
  assert.doesNotMatch(pages[0].html, /data-slot="toolbar-overflow"/);
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
  assert.match(chrome, /\.library-dock-tools/);
  assert.match(chrome, /\.library-dock-status/);
  assert.match(chrome, /\.help-dock-status/);
  assert.match(chrome, /\.library-toolbar/);
  assert.match(chrome, /#library\.page-shell/);
  assert.match(chrome, /#library \.library-empty/);
  assert.match(chrome, /#help\.page-shell/);
  assert.match(chrome, /\.help-dock/);
  assert.match(chrome, /\.help-dock-tools/);
  assert.match(chrome, /\.help-dock-tools \.btn-primary[\s\S]*?width:\s*100%/);
  assert.match(chrome, /#help article/);
  assert.match(pages[3].html, /class="help-dock"/);
  assert.match(pages[3].html, /class="help-diagnostics"/);
  assert.match(
    chrome,
    /#help \[data-diagnostics-preview\][\s\S]*overflow-x:\s*auto/,
  );
  assert.match(
    chrome,
    /#help \[data-diagnostics-preview\][\s\S]*font-family:\s*var\(--font-mono\)/,
  );
  assert.doesNotMatch(pages[3].html, /class="card"/);
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
