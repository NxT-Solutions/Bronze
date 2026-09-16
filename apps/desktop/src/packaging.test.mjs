import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const settings = readFileSync(join(root, "settings.html"), "utf8");

test("webview surfaces have no get-task-allow or network updater", () => {
  for (const doc of [html, settings]) {
    assert.doesNotMatch(doc, /get-task-allow/);
    assert.doesNotMatch(doc, /updater|sparkle|s3\.amazonaws/i);
  }
});
