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

test("local help previews redacted diagnostics and lists human gates", () => {
  assert.match(html, /data-slot="help"/);
  assert.match(html, /data-automatic-upload="false"/);
  assert.match(html, /data-diagnostics-preview/);
  assert.match(html, /data-i18n="help.limitations.humanGates"/);
  assert.match(html, /data-i18n="help.capture.composer"/);
  assert.match(html, /3\.9/);
  assert.match(html, /9\.3/);
  assert.doesNotMatch(html, /fetch\(|xmlhttprequest|https:\/\//i);
  assert.equal(
    en["help.limitations.humanGates"],
    "Human validation remains backlog: stories 3.9, 3.10, 5.5, and 9.3.",
  );
  assert.equal(
    en["help.upload.none"],
    "Bronze never uploads diagnostics automatically.",
  );
});
