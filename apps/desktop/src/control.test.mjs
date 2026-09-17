import assert from "node:assert/strict";
import { test } from "node:test";
import { applyActionStatus, runBusy } from "./control.mjs";

test("runBusy sets aria-busy and restores the control", async () => {
  const el = { disabled: false, attrs: {} };
  el.getAttribute = (name) => el.attrs[name] ?? null;
  el.setAttribute = (name, value) => {
    el.attrs[name] = String(value);
  };
  el.removeAttribute = (name) => {
    delete el.attrs[name];
  };
  const result = await runBusy(el, async () => {
    assert.equal(el.getAttribute("aria-busy"), "true");
    assert.equal(el.disabled, true);
    return "ok";
  });
  assert.equal(result, "ok");
  assert.equal(el.getAttribute("aria-busy"), null);
  assert.equal(el.disabled, false);
});

test("action status copies catalog text and hides when missing", () => {
  const status = { textContent: "", hidden: true };
  const root = {
    querySelector(sel) {
      if (sel === "#action-status") {
        return status;
      }
      if (sel.includes("copy.announce.copied")) {
        return { textContent: "Copied to the clipboard." };
      }
      return null;
    },
  };
  applyActionStatus(root, "copy.announce.copied");
  assert.equal(status.textContent, "Copied to the clipboard.");
  assert.equal(status.hidden, false);
  applyActionStatus(root, "missing");
  assert.equal(status.textContent, "");
  assert.equal(status.hidden, true);
});
