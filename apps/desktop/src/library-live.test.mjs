import assert from "node:assert/strict";
import { test } from "node:test";
import {
  ADR_018_STATUS,
  announceCount,
  QUE_007_COMPLETE,
} from "./library-live.mjs";

test("library search stays a substring placeholder", () => {
  assert.equal(announceCount(0), "0 items");
  assert.equal(announceCount(2), "2 items");
  assert.equal(QUE_007_COMPLETE, false);
  assert.equal(ADR_018_STATUS, "Proposed");
});
