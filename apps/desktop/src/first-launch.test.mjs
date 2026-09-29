import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyFirstLaunchSetup,
  finiteByteCount,
  formatSetupStatus,
  parseSetupDto,
  parseSetupPhase,
  setupProgress,
} from "./first-launch.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "index.html"), "utf8");
const live = readFileSync(join(root, "first-launch.mjs"), "utf8");
const queueLive = readFileSync(join(root, "queue-live.mjs"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

function splashDom() {
  const splash = {
    hidden: true,
  };
  const queue = {
    hidden: false,
    inert: false,
    attrs: {},
    setAttribute(name, value) {
      this.attrs[name] = value;
    },
  };
  const skip = {
    href: "#composer-body",
    setAttribute(name, value) {
      if (name === "href") this.href = value;
    },
  };
  const status = { textContent: "" };
  const error = { hidden: true, textContent: "" };
  const retry = { hidden: true };
  const wrap = { hidden: true };
  const meter = {
    max: 100,
    value: undefined,
    attrs: { "aria-labelledby": "first-launch-status" },
    setAttribute(name, value) {
      this.attrs[name] = value;
      if (name === "max") this.max = Number(value);
      if (name === "value") this.value = Number(value);
    },
    removeAttribute(name) {
      delete this.attrs[name];
      if (name === "value") this.value = undefined;
    },
  };
  const percent = { hidden: true, textContent: "" };
  const nodes = {
    "#first-launch-setup": splash,
    "#quick-panel": queue,
    ".skip-link": skip,
    "#first-launch-status": status,
    "#first-launch-error": error,
    "[data-first-launch-retry]": retry,
    "[data-first-launch-progress]": wrap,
    "#first-launch-progress": meter,
    "[data-first-launch-progress-value]": percent,
  };
  return {
    splash,
    queue,
    skip,
    status,
    error,
    retry,
    wrap,
    meter,
    percent,
    querySelector(sel) {
      return nodes[sel] ?? null;
    },
  };
}

test("splash markup uses status, alert, and catalog copy", () => {
  assert.match(html, /id="first-launch-setup"/);
  assert.match(html, /id="first-launch-status"[^>]*role="status"/);
  assert.match(html, /id="first-launch-error"[^>]*role="alert"/);
  assert.match(html, /data-first-launch-retry/);
  assert.match(html, /class="btn-primary"/);
  assert.match(html, /data-i18n="setup.splash.title"/);
  assert.match(html, /data-i18n="setup.splash.once"/);
  assert.match(html, /data-i18n="setup.splash.checking"/);
  assert.match(html, /data-i18n="setup.splash.retry"/);
  assert.match(html, /id="first-launch-progress"/);
  assert.doesNotMatch(html, /<div\b[^>]*(onclick|role="button")/);
  assert.equal(en["setup.splash.title"], "Setting up Bronze");
  assert.equal(
    en["setup.splash.once"],
    "This only happens the first time to set up the app, and later launches skip this when the title model is already ready.",
  );
  assert.equal(en["setup.splash.checking"], "Checking model…");
  assert.equal(en["setup.splash.downloading"], "Downloading title model…");
  assert.equal(en["setup.splash.finishing"], "Finishing setup…");
  assert.equal(en["setup.splash.retry"], "Retry");
  assert.match(en["setup.splash.checkingNamed"], /\{name\}/);
  assert.match(en["setup.splash.downloadingNamed"], /\{name\}/);
  assert.doesNotMatch(en["setup.splash.once"], /\.\s+[A-Z]/);
});

test("progress binds to real bytes and stays visible without motion", () => {
  assert.equal(parseSetupPhase("downloading"), "downloading");
  assert.equal(finiteByteCount("12"), 12);
  const view = setupProgress({
    needed: true,
    phase: "downloading",
    bytesRead: 50,
    bytesTotal: 200,
  });
  assert.equal(view.visible, true);
  assert.equal(view.determinate, true);
  assert.equal(view.percent, 25);
  assert.equal(view.value, 50);
  assert.equal(view.max, 200);
  const unknown = setupProgress({
    needed: true,
    phase: "checking",
  });
  assert.equal(unknown.visible, true);
  assert.equal(unknown.determinate, false);
  assert.match(
    chrome,
    /\[data-reduce-motion\] \.title-engine-progress progress/,
  );
  assert.match(chrome, /\.first-launch-setup/);
  assert.match(chrome, /--radius-page-card/);
  assert.match(html, /id="first-launch-status"[^>]*role="status"/);
  assert.doesNotMatch(live, /animation:\s*progress/);
});

test("show and hide follow needed, and failure keeps retry", () => {
  const dom = splashDom();
  applyFirstLaunchSetup(dom, {
    needed: true,
    phase: "checking",
    model: "smol-360",
    bytesRead: 10,
    bytesTotal: 40,
  });
  assert.equal(dom.splash.hidden, false);
  assert.equal(dom.queue.hidden, true);
  assert.equal(dom.queue.inert, true);
  assert.equal(dom.skip.href, "#first-launch-status");
  assert.match(dom.status.textContent, /Checking model/);
  assert.equal(dom.wrap.hidden, false);
  assert.equal(dom.meter.value, 10);
  assert.equal(dom.percent.hidden, false);
  assert.equal(dom.retry.hidden, true);

  applyFirstLaunchSetup(dom, {
    needed: true,
    phase: "failed",
    error: "bad_hash",
  });
  assert.equal(dom.splash.hidden, false);
  assert.equal(dom.error.hidden, false);
  assert.match(dom.error.textContent, /checksum/);
  assert.equal(dom.retry.hidden, false);
  assert.equal(dom.wrap.hidden, true);

  applyFirstLaunchSetup(dom, { needed: false, phase: "ready" });
  assert.equal(dom.splash.hidden, true);
  assert.equal(dom.queue.hidden, false);
  assert.equal(dom.queue.inert, false);
  assert.equal(dom.skip.href, "#composer-body");
});

test("queue waits for the setup gate and maps invoke progress", () => {
  assert.match(queueLive, /bindFirstLaunchSetup/);
  assert.match(queueLive, /startQuickPanel/);
  assert.match(queueLive, /first_run_status/);
  assert.match(queueLive, /showChromeWindow\("onboarding"\)/);
  assert.match(live, /first_launch_setup/);
  assert.match(live, /retry_first_launch_setup/);
  assert.match(live, /finish_first_launch_setup/);
  assert.match(live, /first-launch-setup/);
  const dto = parseSetupDto({
    needed: true,
    phase: "downloading",
    model: "qwen-05",
    bytes_read: 3,
    bytes_total: 4,
    complete: false,
  });
  assert.equal(dto.bytesRead, 3);
  assert.equal(dto.bytesTotal, 4);
  assert.match(formatSetupStatus(dto), /Downloading title model/);
});
