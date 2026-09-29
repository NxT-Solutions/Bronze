import { catalogMessage, LOCALE_APPLIED_EVENT } from "./apply-locale.mjs";
import { tauriInvoke, tauriListen } from "./tauri-bridge.mjs";

export const FIRST_LAUNCH_SETUP_EVENT = "first-launch-setup";

const SETUP_PHASES = Object.freeze([
  "checking",
  "downloading",
  "finishing",
  "ready",
  "failed",
]);

const ENGINE_NAME_KEYS = {
  "smol-135": "settings.field.titleModel.engine.smol135",
  "smol-360": "settings.field.titleModel.engine.smol360",
  "qwen-05": "settings.field.titleModel.engine.qwen05",
};

const ENGINE_NAME_FALLBACK = {
  "smol-135": "SmolLM2 135M",
  "smol-360": "SmolLM2 360M",
  "qwen-05": "Qwen2.5 0.5B",
};

const STATUS_FALLBACK = {
  checking: "Checking model…",
  checkingNamed: "Checking model… {name}",
  downloading: "Downloading title model…",
  downloadingNamed: "Downloading title model… {name}",
  finishing: "Finishing setup…",
  finishingNamed: "Finishing setup… {name}",
  progressPercent: "{percent}%",
  "error.generic": "Setup could not finish — try again.",
  "error.download_failed":
    "The title model could not be downloaded — try again.",
  "error.bad_hash":
    "The title model did not match the expected checksum — try again.",
  "error.missing": "The title model is missing — try again.",
  "error.unreadable": "The title model could not be read — try again.",
};

const ERROR_KEYS = new Set([
  "download_failed",
  "bad_hash",
  "missing",
  "unreadable",
]);

export function parseSetupPhase(raw) {
  const phase = String(raw ?? "").trim();
  return SETUP_PHASES.includes(phase) ? phase : "";
}

export function finiteByteCount(value) {
  if (value == null || value === "") {
    return null;
  }
  const n = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(n) || n < 0) {
    return null;
  }
  return n;
}

export function parseSetupDto(raw) {
  const dto = raw && typeof raw === "object" ? raw : {};
  const phase = parseSetupPhase(dto.phase);
  const errorRaw = String(dto.error ?? "").trim();
  return {
    needed: dto.needed === true,
    phase,
    model: String(dto.model ?? "").trim(),
    bytesRead: finiteByteCount(dto.bytesRead ?? dto.bytes_read),
    bytesTotal: finiteByteCount(dto.bytesTotal ?? dto.bytes_total),
    error: ERROR_KEYS.has(errorRaw) ? errorRaw : errorRaw ? "generic" : "",
    complete: dto.complete === true,
  };
}

export function setupProgress(dto) {
  const parsed = parseSetupDto(dto);
  const busy =
    parsed.phase === "checking" ||
    parsed.phase === "downloading" ||
    parsed.phase === "finishing";
  if (!busy) {
    return {
      visible: false,
      determinate: false,
      percent: null,
      value: 0,
      max: 1,
    };
  }
  const total = parsed.bytesTotal;
  const read = parsed.bytesRead;
  if (total != null && total > 0 && read != null) {
    const bounded = Math.min(read, total);
    return {
      visible: true,
      determinate: true,
      percent: Math.floor((bounded * 100) / total),
      value: bounded,
      max: total,
    };
  }
  return {
    visible: true,
    determinate: false,
    percent: null,
    value: 0,
    max: 1,
  };
}

export function setupModelName(id) {
  if (!id) {
    return "";
  }
  const key = ENGINE_NAME_KEYS[id];
  if (!key) {
    return ENGINE_NAME_FALLBACK[id] || "";
  }
  return catalogMessage(key) || ENGINE_NAME_FALLBACK[id] || "";
}

function message(key, fallbackKey) {
  return catalogMessage(key) || STATUS_FALLBACK[fallbackKey] || "";
}

export function formatSetupStatus(dto) {
  const parsed = parseSetupDto(dto);
  if (parsed.phase === "failed") {
    const error = parsed.error || "generic";
    return message(`setup.splash.error.${error}`, `error.${error}`);
  }
  const name = setupModelName(parsed.model);
  const phase = parsed.phase || "checking";
  if (name) {
    return message(`setup.splash.${phase}Named`, `${phase}Named`).replaceAll(
      "{name}",
      name,
    );
  }
  return message(`setup.splash.${phase}`, phase);
}

export function formatSetupPercent(percent) {
  return message("setup.splash.progressPercent", "progressPercent").replaceAll(
    "{percent}",
    String(percent),
  );
}

export function applyFirstLaunchSetup(root, dto) {
  const splash = root.querySelector("#first-launch-setup");
  const queue = root.querySelector("#quick-panel");
  const skip = root.querySelector(".skip-link");
  if (!splash) {
    return parseSetupDto(dto);
  }
  const parsed = parseSetupDto(dto);
  const failed = parsed.phase === "failed";
  const show = parsed.needed;
  splash.hidden = !show;
  if (queue) {
    queue.hidden = show;
    queue.inert = show;
    queue.setAttribute("aria-hidden", show ? "true" : "false");
  }
  if (skip) {
    skip.setAttribute("href", show ? "#first-launch-status" : "#composer-body");
  }
  const status = root.querySelector("#first-launch-status");
  if (status) {
    status.textContent = formatSetupStatus(parsed);
  }
  const error = root.querySelector("#first-launch-error");
  if (error) {
    error.hidden = !failed;
    error.textContent = failed ? formatSetupStatus(parsed) : "";
  }
  const retry = root.querySelector("[data-first-launch-retry]");
  if (retry) {
    retry.hidden = !failed;
  }
  syncSetupProgress(root, parsed, failed);
  return parsed;
}

function syncSetupProgress(root, dto, failed) {
  const wrap = root.querySelector("[data-first-launch-progress]");
  const meter = root.querySelector("#first-launch-progress");
  const percentNode = root.querySelector("[data-first-launch-progress-value]");
  const view = failed
    ? { visible: false, determinate: false, percent: null, value: 0, max: 1 }
    : setupProgress(dto);
  if (wrap) {
    wrap.hidden = !view.visible;
  }
  if (!meter) {
    return;
  }
  if (!view.visible || !view.determinate) {
    if (typeof meter.removeAttribute === "function") {
      meter.removeAttribute("value");
    } else {
      meter.value = undefined;
    }
    if (typeof meter.setAttribute === "function") {
      meter.setAttribute("aria-labelledby", "first-launch-status");
    }
    if (percentNode) {
      percentNode.hidden = true;
      percentNode.textContent = "";
    }
    return;
  }
  meter.max = view.max;
  meter.value = view.value;
  if (typeof meter.setAttribute === "function") {
    meter.setAttribute("max", String(view.max));
    meter.setAttribute("value", String(view.value));
    meter.setAttribute(
      "aria-labelledby",
      "first-launch-status first-launch-progress-value",
    );
  }
  if (percentNode) {
    percentNode.hidden = false;
    percentNode.textContent = formatSetupPercent(view.percent);
  }
}

function hideSplash(root) {
  applyFirstLaunchSetup(root, {
    needed: false,
    phase: "ready",
  });
}

export async function bindFirstLaunchSetup(
  root = globalThis.document,
  invokeFn = tauriInvoke,
  listenFn = tauriListen,
) {
  const splash = root?.querySelector?.("#first-launch-setup");
  if (!splash) {
    return { blocked: false };
  }

  let last = {
    needed: false,
    phase: "ready",
    model: "",
    bytesRead: null,
    bytesTotal: null,
    error: "",
  };
  const readyWaiters = [];

  function paint(raw) {
    last = applyFirstLaunchSetup(root, raw);
    if (!last.needed) {
      for (const resolve of readyWaiters.splice(0)) {
        resolve();
      }
    }
    if (last.needed && last.complete && last.phase !== "failed") {
      finish();
    }
    return last;
  }

  async function refresh() {
    try {
      const dto = await invokeFn("first_launch_setup");
      return paint(dto);
    } catch {
      hideSplash(root);
      last = { needed: false, phase: "ready" };
      for (const resolve of readyWaiters.splice(0)) {
        resolve();
      }
      return last;
    }
  }

  async function finish() {
    try {
      const dto = await invokeFn("finish_first_launch_setup");
      return paint({ ...dto, needed: false, phase: "ready" });
    } catch {
      hideSplash(root);
      last = { needed: false, phase: "ready" };
      return last;
    }
  }

  async function retry() {
    try {
      const dto = await invokeFn("retry_first_launch_setup");
      return paint(dto);
    } catch {
      return paint({
        needed: true,
        phase: "failed",
        error: "generic",
      });
    }
  }

  root
    .querySelector("[data-first-launch-retry]")
    ?.addEventListener("click", () => {
      retry();
    });
  root.addEventListener?.(LOCALE_APPLIED_EVENT, () => {
    if (last.needed) {
      applyFirstLaunchSetup(root, last);
    }
  });
  await listenFn(FIRST_LAUNCH_SETUP_EVENT, (event) => {
    paint(event?.payload ?? event);
  });
  const first = await refresh();
  return {
    blocked: first.needed === true,
    whenReady(fn) {
      if (!first.needed) {
        fn();
        return;
      }
      readyWaiters.push(fn);
    },
    retry,
    finish,
  };
}
