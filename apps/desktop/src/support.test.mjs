import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "help.html"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

test("local help previews redacted diagnostics without automatic upload", () => {
  assert.match(html, /data-slot="help"/);
  assert.match(html, /data-automatic-upload="false"/);
  assert.match(html, /data-diagnostics-preview/);
  assert.match(html, /data-export-support/);
  assert.match(html, /data-i18n="help.capture.composer"/);
  assert.match(html, /3\.9/);
  assert.match(html, /9\.3/);
  assert.doesNotMatch(html, /fetch\(|xmlhttprequest|https:\/\//i);
  assert.equal(en["help.diagnostics.export"], "Export Support File…");
  assert.equal(
    en["help.diagnostics.export.explain"],
    "Save a support file on this Mac and attach it to a GitHub issue or pull request; Bronze never uploads it.",
  );
  assert.equal(
    en["help.upload.none"],
    "Bronze does not upload diagnostics or the support file automatically, so everything stays on this Mac.",
  );
  assert.equal(
    en["help.capture.composer"],
    "Select text in any app including Bronze, then Capture from the menu bar, use the Capture selection shortcut from Settings, or type into the composer when capture is unavailable.",
  );
  assert.equal(
    html.match(/data-i18n="help.capture.composer"[^>]*>\s*([^<]+?)\s*</)?.[1],
    en["help.capture.composer"],
  );
});
