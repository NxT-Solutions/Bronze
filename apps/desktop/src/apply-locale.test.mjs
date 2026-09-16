import assert from "node:assert/strict";
import { test } from "node:test";
import {
  catalogHasRealSpaces,
  HAND_TEST_DEFAULT_LOCALE,
  resolveUiLocale,
} from "./apply-locale.mjs";

test("hand-test locale stays en unless a pseudo locale is explicit", () => {
  assert.equal(HAND_TEST_DEFAULT_LOCALE, "en");
  assert.equal(resolveUiLocale("system"), "en");
  assert.equal(resolveUiLocale("en"), "en");
  assert.equal(resolveUiLocale("en-XA"), "en-XA");
  assert.equal(resolveUiLocale(undefined), "en");
  assert.equal(
    catalogHasRealSpaces("Needed for the global capture chord"),
    true,
  );
  assert.equal(
    catalogHasRealSpaces("Needededforthegloballcaptureechordd"),
    false,
  );
});
