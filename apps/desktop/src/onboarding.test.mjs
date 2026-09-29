import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  applyOnboardingPhase,
  applySplashProgress,
  bindOnboarding,
  COMPLETE_ONBOARDING_COMMAND,
  FIRST_RUN_STATUS_COMMAND,
  isLastWizardStep,
  nextWizardIndex,
  phaseAfterStatus,
  previousWizardIndex,
  splashBusy,
  splashCanContinue,
  splashNeedsRetry,
  TITLE_ENGINE_STATUS_COMMAND,
  TITLE_ENGINE_STATUS_EVENT,
  WIZARD_STEPS,
} from "./onboarding.mjs";
import { RETEST_COMMAND } from "./permission-health.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const html = readFileSync(join(root, "onboarding.html"), "utf8");
const live = readFileSync(join(root, "onboarding.mjs"), "utf8");
const chrome = readFileSync(join(root, "chrome.css"), "utf8");
const en = JSON.parse(
  readFileSync(
    join(root, "../../../packages/i18n/locales/en/app.json"),
    "utf8",
  ),
);

function el(init = {}) {
  return {
    hidden: false,
    textContent: "",
    dataset: {},
    attrs: {},
    listeners: {},
    setAttribute(name, value) {
      this.attrs[name] = value;
    },
    getAttribute(name) {
      return this.attrs[name];
    },
    removeAttribute(name) {
      delete this.attrs[name];
    },
    addEventListener(name, fn) {
      this.listeners[name] = fn;
    },
    ...init,
  };
}

function onboardingRoot() {
  const splash = el({ hidden: true });
  const wizard = el();
  const steps = Object.fromEntries(WIZARD_STEPS.map((name) => [name, el()]));
  const back = el({ hidden: true });
  const next = el();
  const status = el({ hidden: true });
  const splashStatus = el();
  const progress = el({ hidden: true });
  const bar = el({ max: 100, value: 0 });
  const percent = el();
  const retry = el({ hidden: true });
  const nodes = {
    "#onboarding": null,
    '[data-onboarding-pane="splash"]': splash,
    '[data-onboarding-pane="wizard"]': wizard,
    "[data-onboarding-back]": back,
    "[data-onboarding-next]": next,
    "[data-onboarding-status]": status,
    "[data-onboarding-splash-status]": splashStatus,
    "[data-onboarding-splash-progress]": progress,
    "#onboarding-splash-bar": bar,
    "[data-onboarding-splash-percent]": percent,
    "[data-onboarding-splash-retry]": retry,
  };
  for (const name of WIZARD_STEPS) {
    nodes[`[data-onboarding-step="${name}"]`] = steps[name];
  }
  const main = {
    dataset: {},
    splash,
    wizard,
    steps,
    back,
    next,
    status,
    splashStatus,
    progress,
    bar,
    percent,
    retry,
    querySelector(sel) {
      return nodes[sel] ?? null;
    },
  };
  nodes["#onboarding"] = main;
  return main;
}

test("wizard markup uses catalog stills and live permission health", () => {
  assert.match(html, /id="onboarding"/);
  assert.match(html, /href="#onboarding-title"/);
  assert.match(html, /id="onboarding-title"[^>]*tabindex="-1"/);
  assert.match(html, /data-onboarding-pane="splash"/);
  assert.match(html, /role="status"/);
  assert.match(html, /onboarding-media\/bronze-hero\.png/);
  assert.match(html, /onboarding-media\/bronze-queue\.png/);
  assert.match(html, /onboarding-media\/bronze-settings\.png/);
  assert.match(html, /data-onboarding-step="welcome"/);
  assert.match(html, /data-onboarding-step="capture"/);
  assert.match(html, /data-onboarding-step="queue"/);
  assert.match(html, /data-onboarding-step="settings"/);
  assert.match(html, /data-onboarding-step="permissions"/);
  assert.match(html, /data-permission-retest="accessibility"/);
  assert.match(html, /data-permission-retest="inputMonitoring"/);
  assert.match(html, /data-permission-allow-notifications/);
  assert.match(html, /data-permission-open-settings="accessibility"/);
  assert.match(html, /data-permission-cdhash/);
  assert.match(html, /settings\.permission\.identityDrift/);
  assert.match(html, /class="btn-primary"/);
  assert.match(html, /class="btn-ghost"/);
  assert.doesNotMatch(html, /<div\b[^>]*(onclick|role="button")/);
  assert.doesNotMatch(html.toLowerCase(), /copper|cooper/);
  assert.deepEqual(WIZARD_STEPS, [
    "welcome",
    "capture",
    "queue",
    "settings",
    "permissions",
  ]);
  assert.equal(
    en["onboarding.splash.body"],
    "This is only the first time to set up the app.",
  );
  assert.doesNotMatch(en["onboarding.splash.body"], /\.\s+[A-Z]/);
  assert.equal(en["onboarding.finish"], "Finish setup");
  assert.match(chrome, /#onboarding\.page-shell/);
  assert.match(chrome, /\.onboarding-dock/);
  assert.match(chrome, /\.onboarding-shot/);
  assert.match(live, /RETEST_COMMAND/);
  assert.match(live, /requestNoticeAuthorization/);
  assert.match(live, /complete_onboarding/);
});

test("splash then wizard then finish, and later launches skip both", () => {
  assert.equal(phaseAfterStatus({ onboardingComplete: true }), "done");
  assert.equal(
    phaseAfterStatus({ onboardingComplete: false, needsSplash: true }),
    "splash",
  );
  assert.equal(
    phaseAfterStatus({ onboardingComplete: false, needsSplash: false }),
    "wizard",
  );
  assert.equal(splashBusy({ titlePhase: "loading" }), true);
  assert.equal(splashBusy({ titlePhase: "downloading" }), true);
  assert.equal(splashCanContinue({ titlePhase: "loading" }), false);
  assert.equal(splashCanContinue({ titlePhase: "ready" }), true);
  assert.equal(splashCanContinue({ titlePhase: "failed" }), true);
  assert.equal(splashNeedsRetry({ titlePhase: "missing" }), true);
  assert.equal(nextWizardIndex(0), 1);
  assert.equal(previousWizardIndex(0), 0);
  assert.equal(isLastWizardStep(WIZARD_STEPS.length - 1), true);
});

test("progress uses real bytes and hides continue while the model loads", () => {
  const tree = onboardingRoot();
  applyOnboardingPhase(tree, "splash", { titlePhase: "loading" });
  assert.equal(tree.dataset.phase, "splash");
  assert.equal(tree.splash.hidden, false);
  assert.equal(tree.wizard.hidden, true);
  assert.equal(tree.next.hidden, true);
  applySplashProgress(tree, {
    titlePhase: "loading",
    bytesRead: 25,
    bytesTotal: 100,
  });
  assert.equal(tree.progress.hidden, false);
  assert.equal(tree.bar.value, 25);
  assert.equal(tree.percent.textContent, "25%");
  applyOnboardingPhase(tree, "splash", { titlePhase: "failed" });
  assert.equal(tree.next.hidden, false);
  applySplashProgress(tree, { titlePhase: "failed" });
  assert.equal(tree.retry.hidden, false);
});

test("wizard steps and finish invoke complete_onboarding", async () => {
  const tree = onboardingRoot();
  const calls = [];
  const invokeFn = async (cmd) => {
    calls.push(cmd);
    if (cmd === FIRST_RUN_STATUS_COMMAND) {
      return {
        onboardingComplete: false,
        needsWizard: true,
        needsSplash: false,
        titlePhase: "ready",
      };
    }
    return {};
  };
  const session = await bindOnboarding(tree, invokeFn, async () => {});
  assert.equal(session.status.needsSplash, false);
  assert.equal(tree.steps.welcome.hidden, false);
  assert.equal(tree.steps.permissions.hidden, true);
  await tree.next.listeners.click();
  await tree.next.listeners.click();
  await tree.next.listeners.click();
  await tree.next.listeners.click();
  assert.equal(session.stepIndex, 4);
  assert.equal(tree.steps.permissions.hidden, false);
  assert.equal(tree.next.attrs["data-i18n"], "onboarding.finish");
  assert.ok(calls.includes(RETEST_COMMAND));
  await tree.next.listeners.click();
  assert.ok(calls.includes(COMPLETE_ONBOARDING_COMMAND));
  assert.ok(calls.includes(FIRST_RUN_STATUS_COMMAND));
  assert.equal(TITLE_ENGINE_STATUS_COMMAND, "title_engine_status");
  assert.equal(TITLE_ENGINE_STATUS_EVENT, "title-engine-status");
});
