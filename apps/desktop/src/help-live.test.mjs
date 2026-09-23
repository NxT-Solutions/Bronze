import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  refreshSupportPreview,
  showSupportExportStatus,
  supportExportFailureCode,
} from "./help-live.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const live = readFileSync(join(root, "help-live.mjs"), "utf8");
const html = readFileSync(join(root, "help.html"), "utf8");

test("help export invokes rust picker and never sends a path", () => {
  assert.match(html, /data-export-support/);
  assert.match(html, /data-export-support-status/);
  assert.match(html, /help-live\.mjs/);
  assert.match(live, /export_support_file/);
  assert.match(live, /preview_support_bundle/);
  assert.match(live, /requestedPath: null/);
  assert.doesNotMatch(live, /fetch\(|xmlhttprequest|https:\/\//i);
  assert.doesNotMatch(html, /fetch\(|xmlhttprequest|https:\/\//i);
  assert.equal(supportExportFailureCode("picker_cancelled"), "");
  assert.equal(
    supportExportFailureCode("picker_unavailable"),
    "picker_unavailable",
  );
  assert.equal(supportExportFailureCode("nope"), "support_path_invalid");
  const status = { textContent: "", hidden: true };
  const fakeRoot = {
    querySelector(sel) {
      return sel.includes("data-export-support-status") ? status : null;
    },
  };
  showSupportExportStatus(fakeRoot, "help.diagnostics.export.done");
  assert.equal(status.textContent, "Saved the support file on this Mac.");
  assert.equal(status.hidden, false);
  showSupportExportStatus(fakeRoot, "");
  assert.equal(status.textContent, "");
  assert.equal(status.hidden, true);
});

test("help preview is visible and uses the rust report text", async () => {
  assert.doesNotMatch(html, /events=0/);
  const preview = {
    textContent: "fallback",
    hidden: true,
    removeAttribute(name) {
      if (name === "aria-hidden") {
        this.ariaHidden = false;
      }
    },
  };
  const root = {
    querySelector(sel) {
      return sel.includes("data-diagnostics-preview") ? preview : null;
    },
  };
  await refreshSupportPreview(
    root,
    async () => "Bronze support report\nversion=0.1.0\nos=macos",
  );
  assert.equal(preview.hidden, false);
  assert.match(preview.textContent, /Bronze support report/);
  assert.match(preview.textContent, /version=0.1.0/);
});
