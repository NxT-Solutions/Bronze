import { catalogMessage } from "./apply-locale.mjs";
import {
  applyPermissionResult,
  RETEST_COMMAND,
  requestNoticeAuthorization,
} from "./permission-health.mjs";
import { tauriInvoke, tauriListen } from "./tauri-bridge.mjs";

export const FIRST_RUN_STATUS_COMMAND = "first_run_status";
export const COMPLETE_ONBOARDING_COMMAND = "complete_onboarding";
export const TITLE_ENGINE_STATUS_COMMAND = "title_engine_status";
export const TITLE_ENGINE_STATUS_EVENT = "title-engine-status";

export const WIZARD_STEPS = [
  "welcome",
  "capture",
  "queue",
  "settings",
  "permissions",
];

const SPLASH_STATUS_KEYS = {
  loading: "onboarding.splash.status",
  hashing: "onboarding.splash.status",
  ready: "onboarding.splash.status.ready",
  missing: "onboarding.splash.status.missing",
  failed: "onboarding.splash.status.failed",
  idle: "onboarding.splash.status",
};

const SPLASH_STATUS_FALLBACK = {
  loading: "Preparing the title model",
  hashing: "Preparing the title model",
  ready: "The title model is ready.",
  missing: "The title model is not on this Mac yet.",
  failed: "The title model could not load, and you can try again.",
  idle: "Preparing the title model",
};

export function phaseAfterStatus(status) {
  if (status?.onboardingComplete) {
    return "done";
  }
  if (status?.needsSplash) {
    return "splash";
  }
  return "wizard";
}

export function splashBusy(status) {
  const phase = String(status?.titlePhase || status?.title_phase || "");
  return phase === "loading" || phase === "hashing" || phase === "downloading";
}

export function splashCanContinue(status) {
  return !splashBusy(status);
}

export function splashNeedsRetry(status) {
  const phase = String(status?.titlePhase || status?.title_phase || "");
  return phase === "failed" || phase === "missing";
}

export function nextWizardIndex(index) {
  return Math.min(index + 1, WIZARD_STEPS.length - 1);
}

export function previousWizardIndex(index) {
  return Math.max(index - 1, 0);
}

export function isLastWizardStep(index) {
  return index >= WIZARD_STEPS.length - 1;
}

function titlePhase(status) {
  return String(status?.titlePhase || status?.title_phase || "idle");
}

function byteCount(status, camel, snake) {
  const value = status?.[camel] ?? status?.[snake];
  const n = Number(value);
  return Number.isFinite(n) && n >= 0 ? n : null;
}

export function applyOnboardingPhase(root, phase, status) {
  if (!root) {
    return;
  }
  if (root.dataset) {
    root.dataset.phase = phase;
  }
  const splash = root.querySelector('[data-onboarding-pane="splash"]');
  const wizard = root.querySelector('[data-onboarding-pane="wizard"]');
  if (splash) {
    splash.hidden = phase !== "splash";
  }
  if (wizard) {
    wizard.hidden = phase !== "wizard";
  }
  const next = root.querySelector("[data-onboarding-next]");
  const back = root.querySelector("[data-onboarding-back]");
  if (phase === "splash") {
    if (back) {
      back.hidden = true;
    }
    if (next) {
      next.hidden = !splashCanContinue(status);
      const key = "onboarding.next";
      next.setAttribute("data-i18n", key);
      next.textContent = catalogMessage(key) || "Continue";
    }
  } else if (next) {
    next.hidden = false;
  }
}

export function applyWizardStep(root, index) {
  const clamped = Math.max(0, Math.min(index, WIZARD_STEPS.length - 1));
  for (const name of WIZARD_STEPS) {
    const step = root.querySelector(`[data-onboarding-step="${name}"]`);
    if (step) {
      step.hidden = name !== WIZARD_STEPS[clamped];
    }
  }
  const back = root.querySelector("[data-onboarding-back]");
  if (back) {
    back.hidden = clamped === 0;
  }
  const next = root.querySelector("[data-onboarding-next]");
  if (next) {
    const last = isLastWizardStep(clamped);
    const key = last ? "onboarding.finish" : "onboarding.next";
    next.setAttribute("data-i18n", key);
    next.textContent =
      catalogMessage(key) || (last ? "Finish setup" : "Continue");
  }
  return clamped;
}

export function applySplashProgress(root, status) {
  const phase = titlePhase(status);
  const statusEl = root.querySelector("[data-onboarding-splash-status]");
  if (statusEl) {
    const key = SPLASH_STATUS_KEYS[phase] || SPLASH_STATUS_KEYS.loading;
    statusEl.setAttribute("data-i18n", key);
    statusEl.textContent =
      catalogMessage(key) ||
      SPLASH_STATUS_FALLBACK[phase] ||
      SPLASH_STATUS_FALLBACK.loading;
  }
  const wrap = root.querySelector("[data-onboarding-splash-progress]");
  const bar = root.querySelector("#onboarding-splash-bar");
  const percent = root.querySelector("[data-onboarding-splash-percent]");
  const read = byteCount(status, "bytesRead", "bytes_read");
  const total = byteCount(status, "bytesTotal", "bytes_total");
  const determinate = read != null && total != null && total > 0;
  const busy = splashBusy(status);
  if (wrap) {
    wrap.hidden = !(busy || determinate);
  }
  if (bar) {
    if (determinate) {
      bar.removeAttribute("aria-busy");
      bar.max = 100;
      bar.value = Math.min(100, Math.round((read / total) * 100));
    } else if (busy) {
      bar.removeAttribute("value");
      bar.setAttribute("aria-busy", "true");
    }
  }
  if (percent) {
    percent.textContent = determinate && bar ? `${bar.value}%` : "";
  }
  const retry = root.querySelector("[data-onboarding-splash-retry]");
  if (retry) {
    retry.hidden = !splashNeedsRetry(status);
  }
}

export async function requestOnboardingPermissions(
  root,
  invokeFn = tauriInvoke,
) {
  try {
    const result = await invokeFn(RETEST_COMMAND);
    applyPermissionResult(root, result);
  } catch {
    // Retest buttons stay available if the first OS prompt does not run.
  }
  try {
    await requestNoticeAuthorization(root, invokeFn);
  } catch {
    // Allow notifications stays available if the request is not determined.
  }
}

export async function bindOnboarding(
  root = document,
  invokeFn = tauriInvoke,
  listenFn = tauriListen,
) {
  const main = root.querySelector?.("#onboarding") ?? root;
  let stepIndex = 0;
  let askedPermissions = false;
  let current = {
    onboardingComplete: false,
    needsWizard: true,
    needsSplash: false,
    titlePhase: "idle",
  };

  const paint = (status) => {
    current = { ...current, ...status };
    const phase = phaseAfterStatus(current);
    applyOnboardingPhase(main, phase, current);
    if (phase === "splash") {
      applySplashProgress(main, current);
    } else {
      stepIndex = applyWizardStep(main, stepIndex);
      maybeAsk();
    }
    return phase;
  };

  const maybeAsk = () => {
    if (WIZARD_STEPS[stepIndex] === "permissions" && !askedPermissions) {
      askedPermissions = true;
      requestOnboardingPermissions(main, invokeFn);
    }
  };

  try {
    const status = await invokeFn(FIRST_RUN_STATUS_COMMAND);
    paint(status);
  } catch {
    paint(current);
  }

  listenFn(TITLE_ENGINE_STATUS_EVENT, (event) => {
    const payload = event?.payload ?? event ?? {};
    const next = {
      ...current,
      titlePhase: payload.phase ?? current.titlePhase,
      bytesRead: payload.bytesRead ?? payload.bytes_read,
      bytesTotal: payload.bytesTotal ?? payload.bytes_total,
    };
    if (
      current.needsSplash &&
      splashCanContinue(next) &&
      titlePhase(next) === "ready"
    ) {
      next.needsSplash = false;
    }
    paint(next);
  });

  main
    .querySelector("[data-onboarding-splash-retry]")
    ?.addEventListener("click", async () => {
      try {
        const status = await invokeFn(TITLE_ENGINE_STATUS_COMMAND);
        paint({
          ...current,
          titlePhase: status.phase ?? status.titlePhase,
          bytesRead: status.bytesRead ?? status.bytes_read,
          bytesTotal: status.bytesTotal ?? status.bytes_total,
        });
      } catch {
        paint({ ...current, titlePhase: "failed" });
      }
    });

  main
    .querySelector("[data-onboarding-back]")
    ?.addEventListener("click", () => {
      stepIndex = applyWizardStep(main, previousWizardIndex(stepIndex));
      maybeAsk();
    });

  main
    .querySelector("[data-onboarding-next]")
    ?.addEventListener("click", async () => {
      if (phaseAfterStatus(current) === "splash") {
        if (!splashCanContinue(current)) {
          return;
        }
        current = { ...current, needsSplash: false };
        paint(current);
        return;
      }
      if (!isLastWizardStep(stepIndex)) {
        stepIndex = applyWizardStep(main, nextWizardIndex(stepIndex));
        maybeAsk();
        return;
      }
      try {
        await invokeFn(COMPLETE_ONBOARDING_COMMAND);
      } catch {
        const statusEl = main.querySelector("[data-onboarding-status]");
        if (statusEl) {
          statusEl.hidden = false;
          statusEl.textContent =
            catalogMessage("onboarding.finish.failed") ||
            "Setup could not be marked complete.";
        }
      }
    });

  return {
    get stepIndex() {
      return stepIndex;
    },
    get status() {
      return current;
    },
  };
}

if (globalThis.document?.querySelector?.("#onboarding")) {
  bindOnboarding();
}
