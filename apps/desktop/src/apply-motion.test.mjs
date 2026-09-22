import assert from "node:assert/strict";
import { test } from "node:test";
import {
  applyMotionDataset,
  parseReduceMotion,
  reduceMotionFromSettings,
  resolveMotionDataset,
} from "./apply-motion.mjs";
import { motionAllowed } from "./control.mjs";

test("reduce-motion setting maps to data-motion and can force full motion", () => {
  assert.equal(parseReduceMotion("system"), "system");
  assert.equal(parseReduceMotion("on"), "on");
  assert.equal(parseReduceMotion("reduce"), "on");
  assert.equal(parseReduceMotion("off"), "off");
  assert.equal(parseReduceMotion("nope"), "system");
  assert.equal(
    reduceMotionFromSettings({ accessibility: { motion: "reduce" } }),
    "on",
  );
  assert.equal(
    reduceMotionFromSettings({
      general: { reduceMotion: "off" },
      accessibility: { motion: "on" },
    }),
    "off",
  );
  assert.equal(resolveMotionDataset("off", true), "full");
  assert.equal(resolveMotionDataset("on", false), "reduce");
  assert.equal(resolveMotionDataset("system", true), "reduce");
  assert.equal(resolveMotionDataset("system", false), "full");

  const attrs = new Set();
  const html = {
    dataset: {},
    toggleAttribute(name, on) {
      if (on) {
        attrs.add(name);
      } else {
        attrs.delete(name);
      }
    },
  };
  assert.equal(applyMotionDataset(html, "off", true), "full");
  assert.equal(html.dataset.motion, "full");
  assert.equal(attrs.has("data-reduce-motion"), false);
  assert.equal(applyMotionDataset(html, "on", false), "reduce");
  assert.equal(html.dataset.motion, "reduce");
  assert.equal(attrs.has("data-reduce-motion"), true);

  assert.equal(
    motionAllowed({
      documentElement: {
        dataset: { motion: "full" },
        hasAttribute: () => true,
      },
      defaultView: { matchMedia: () => ({ matches: true }) },
    }),
    true,
  );
  assert.equal(
    motionAllowed({
      documentElement: {
        dataset: { motion: "reduce" },
        hasAttribute: () => false,
      },
      defaultView: { matchMedia: () => ({ matches: false }) },
    }),
    false,
  );
});
