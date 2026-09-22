import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const library = readFileSync(join(root, "library.html"), "utf8");
const settings = readFileSync(join(root, "settings.html"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

test("library owns backup export import with secret warning and no off schedule", () => {
  assert.match(library, /data-i18n="library.backup.now"/);
  assert.match(library, /data-restore-preview/);
  assert.match(library, /data-export-secret-warning/);
  assert.match(library, /data-i18n="export.preview.secretBodies"/);
  assert.doesNotMatch(
    settings,
    /id="backup-schedule"[\s\S]*?<option value="off"/,
  );
  assert.equal(en["library.backup.now"], "Back Up Now");
  assert.equal(
    en["export.preview.secretBodies"],
    "Item bodies in this export may contain secrets.",
  );
});
