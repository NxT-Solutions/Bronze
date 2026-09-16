import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const library = readFileSync(join(root, "library.html"), "utf8");

test("search announces count and never puts the query in diagnostics", () => {
  assert.match(library, /id="library-search"/);
  assert.match(library, /id="search-count"[^>]*role="status"/);
  assert.match(library, /data-i18n="queue.count"/);
  assert.doesNotMatch(library, /data-diagnostic[^>]*q=/);
  assert.doesNotMatch(library, /diagnostic.*query/i);
});
