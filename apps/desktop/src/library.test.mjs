import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const quick = readFileSync(join(root, "index.html"), "utf8");
const library = readFileSync(join(root, "library.html"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

test("library window paginates and archives; quick cannot import export backup", () => {
  assert.match(quick, /id="active-section"/);
  assert.match(quick, /data-i18n="panel.section.active"/);
  assert.doesNotMatch(quick, /data-i18n="(import|export|backup|library\.)/);
  assert.doesNotMatch(library, /data-i18n="(import|export|backup)/);
  assert.match(library, /id="library"/);
  assert.match(library, /data-library-state="ready"/);
  assert.match(library, /data-i18n="library.archive"/);
  assert.match(library, /data-i18n="library.paginate"/);
  assert.match(library, /<search\b/);
  assert.equal(en["library.title"], "Library");
  assert.equal(en["library.archive"], "Archive");
  assert.equal(en["library.paginate"], "Next page");
});
