import {
  applyHandTestLocale,
  catalogMessage,
  emitUiLocaleChanged,
  LOCALE_APPLIED_EVENT,
} from "./apply-locale.mjs";
import {
  applyMotionFromSettings,
  emitMotionChanged,
  parseReduceMotion,
} from "./apply-motion.mjs";
import { runBusy } from "./control.mjs";
import { sourceIconSrc } from "./item-view.mjs";
import {
  emitQueueSortChanged,
  parseQueueSort,
  queueSortChange,
} from "./queue-sort.mjs";
import { bindShortcutRegistry } from "./shortcuts.mjs";
import { showChromeWindow, tauriInvoke, tauriListen } from "./tauri-bridge.mjs";

const SWITCHER_LOCALES = [
  "en",
  "nl",
  "fr",
  "de",
  "es",
  "it",
  "ru",
  "uk",
  "hr",
  "sl",
  "da",
  "sv",
  "nb",
  "fi",
  "tr",
];

export const TITLE_MODEL_IDS = Object.freeze([
  "extractive",
  "smol-135",
  "smol-360",
  "qwen-05",
  "custom",
  "ollama",
  "hosted-openai",
  "hosted-anthropic",
  "hosted-openrouter",
]);

const BUNDLED_TITLE_MODEL_IDS = Object.freeze([
  "smol-135",
  "smol-360",
  "qwen-05",
]);

const LOCAL_TITLE_MODEL_IDS = Object.freeze([
  "extractive",
  ...BUNDLED_TITLE_MODEL_IDS,
  "custom",
]);

const HOSTED_TITLE_MODEL_IDS = Object.freeze([
  "hosted-openai",
  "hosted-anthropic",
  "hosted-openrouter",
]);

const INTEGRATION_TITLE_MODEL_IDS = Object.freeze([
  "ollama",
  ...HOSTED_TITLE_MODEL_IDS,
]);

const TITLE_MODEL_VENDOR = {
  "smol-135": "sh crates/bronze-title-model/scripts/vendor-gguf.sh smol-135",
  "smol-360": "sh crates/bronze-title-model/scripts/vendor-gguf.sh smol-360",
  "qwen-05": "sh crates/bronze-title-model/scripts/vendor-gguf.sh qwen-05",
};

export const TITLE_ENGINE_STATUS_EVENT = "title-engine-status";

const titleEngineRevisions = new WeakMap();
const titleEnginePhases = new WeakMap();

function titleEngineRevision(root) {
  return titleEngineRevisions.get(root) ?? 0;
}

function notedTitleEnginePhase(root) {
  return titleEnginePhases.get(root) ?? "";
}

function rememberTitleEnginePhase(root, phase) {
  titleEnginePhases.set(root, phase);
}

function bumpTitleEngineRevision(root, phase) {
  const next = titleEngineRevision(root) + 1;
  titleEngineRevisions.set(root, next);
  rememberTitleEnginePhase(root, phase);
  return next;
}

const TITLE_ENGINE_PHASES = Object.freeze([
  "idle",
  "loading",
  "hashing",
  "ready",
  "missing",
  "failed",
]);

const TITLE_MODEL_STATUS_FALLBACK = {
  extractive: "Extractive titles use no model file.",
  present: "This file is on this Mac and can title the next capture.",
  missing:
    "Vendored file missing — titles stay extractive until you run {command}.",
  notInBuild: "This model is not in this build, so titles stay extractive.",
  progressPercent: "{percent}%",
  unavailable: "Title engines could not be listed.",
  loading: "Loading {engine}…",
  hashing: "Checking {engine}…",
  ready: "Ready",
  loaded: "Loaded — will title the next capture",
  "failed.bad_hash":
    "This file did not match the expected hash, so titles stay extractive.",
  "failed.timeout": "Loading timed out, so titles stay extractive.",
  "failed.unreadable":
    "This file could not be read, so titles stay extractive.",
  customEmpty: "Import a GGUF to use it for titles.",
  customPresent: "This imported file can title the next capture.",
  customOption: "Imported GGUF — RAM follows the file, 2 CPU threads",
  customOptionSized: "Imported GGUF — about {size} of RAM, 2 CPU threads",
  imported: "Imported {name} ({size}).",
  importFailed: "That file was not a GGUF, so nothing was imported.",
  ollamaStatus: "Ollama will title the next capture when it is running.",
  ollamaEmpty:
    "Ollama is not running — the bundled engine will title this capture.",
  ollamaUnavailable: "Ollama models could not be listed.",
  hostedStatus: "The next capture sends truncated text to {host}.",
  hostedUnconfirmed: "Confirm the host before a capture can send text.",
  hostedBlocked: "That host is not allowed.",
  disclosure:
    "This send goes to {host} with truncated capture text of 2048 characters plus the fixed title instruction.",
};

const TITLE_ENGINE_NAME_KEYS = {
  "smol-135": "settings.field.titleModel.engine.smol135",
  "smol-360": "settings.field.titleModel.engine.smol360",
  "qwen-05": "settings.field.titleModel.engine.qwen05",
  custom: "settings.field.titleModel.engine.custom",
  ollama: "settings.field.titleModel.engine.ollama",
  "hosted-openai": "settings.field.titleModel.engine.hosted",
  "hosted-anthropic": "settings.field.titleModel.engine.hosted",
  "hosted-openrouter": "settings.field.titleModel.engine.hosted",
};

const TITLE_ENGINE_NAME_FALLBACK = {
  "smol-135": "SmolLM2 135M",
  "smol-360": "SmolLM2 360M",
  "qwen-05": "Qwen2.5 0.5B",
  custom: "Imported GGUF",
  ollama: "Ollama",
  "hosted-openai": "Hosted",
  "hosted-anthropic": "Hosted",
  "hosted-openrouter": "Hosted",
};

export function parseTitleModelId(raw) {
  const value = String(raw ?? "").trim();
  return TITLE_MODEL_IDS.includes(value) ? value : "";
}

export function isBundledTitleModel(id) {
  return BUNDLED_TITLE_MODEL_IDS.includes(id);
}

export function isLocalTitleModel(id) {
  return LOCAL_TITLE_MODEL_IDS.includes(id);
}

export function isHostedTitleModel(id) {
  return HOSTED_TITLE_MODEL_IDS.includes(id);
}

export function isIntegrationTitleModel(id) {
  return INTEGRATION_TITLE_MODEL_IDS.includes(id);
}

export function selectedTitleModelId(root, settings = {}) {
  const integration = parseTitleModelId(
    root.querySelector("#title-integration")?.value,
  );
  if (isIntegrationTitleModel(integration)) {
    return integration;
  }
  return (
    parseTitleModelId(root.querySelector("#title-model")?.value) ||
    parseTitleModelId(settings?.general?.titleModel)
  );
}

export function formatTitleFileSize(bytes) {
  const n = Number(bytes);
  if (!Number.isFinite(n) || n < 0) {
    return "0 B";
  }
  if (n >= 1024 * 1024) {
    return `${Math.round(n / (1024 * 1024))} MB`;
  }
  if (n >= 1024) {
    return `${Math.round(n / 1024)} KB`;
  }
  return `${Math.round(n)} B`;
}

export function formatCustomTitleOption(bytes) {
  const n = Number(bytes);
  if (Number.isFinite(n) && n > 0) {
    const template =
      catalogMessage("settings.field.titleModel.custom.sized") ||
      TITLE_MODEL_STATUS_FALLBACK.customOptionSized;
    return template.replaceAll("{size}", formatTitleFileSize(n));
  }
  return (
    catalogMessage("settings.field.titleModel.custom") ||
    TITLE_MODEL_STATUS_FALLBACK.customOption
  );
}

export function formatTitleModelStatus(row, options = {}) {
  const id = parseTitleModelId(row?.id) || "extractive";
  if (id === "extractive") {
    return (
      catalogMessage("settings.field.titleModel.extractive.status") ||
      TITLE_MODEL_STATUS_FALLBACK.extractive
    );
  }
  if (id === "custom") {
    if (row?.present || row?.displayName) {
      return (
        catalogMessage("settings.field.titleModel.custom.status") ||
        TITLE_MODEL_STATUS_FALLBACK.customPresent
      );
    }
    return (
      catalogMessage("settings.field.titleModel.custom.empty") ||
      TITLE_MODEL_STATUS_FALLBACK.customEmpty
    );
  }
  if (id === "ollama") {
    if (options.unavailable) {
      return (
        catalogMessage("settings.field.titleModel.ollama.unavailable") ||
        TITLE_MODEL_STATUS_FALLBACK.ollamaUnavailable
      );
    }
    return (
      catalogMessage("settings.field.titleModel.ollama.status") ||
      TITLE_MODEL_STATUS_FALLBACK.ollamaStatus
    );
  }
  if (isHostedTitleModel(id)) {
    if (options.blocked) {
      return (
        catalogMessage("settings.field.titleModel.hosted.blocked") ||
        TITLE_MODEL_STATUS_FALLBACK.hostedBlocked
      );
    }
    if (options.unconfirmed) {
      return (
        catalogMessage("settings.field.titleModel.hosted.unconfirmed") ||
        TITLE_MODEL_STATUS_FALLBACK.hostedUnconfirmed
      );
    }
    const template =
      catalogMessage("settings.field.titleModel.hosted.status") ||
      TITLE_MODEL_STATUS_FALLBACK.hostedStatus;
    return template.replaceAll("{host}", options.host || "host");
  }
  if (options.unavailable) {
    return (
      catalogMessage("settings.field.titleModel.unavailable") ||
      TITLE_MODEL_STATUS_FALLBACK.unavailable
    );
  }
  if (row?.present) {
    return (
      catalogMessage("settings.field.titleModel.present") ||
      TITLE_MODEL_STATUS_FALLBACK.present
    );
  }
  if (isBundledTitleModel(id)) {
    return notInBuildStatus();
  }
  const command = String(
    row?.vendorCommand || TITLE_MODEL_VENDOR[id] || "",
  ).trim();
  const template =
    catalogMessage("settings.field.titleModel.missing") ||
    TITLE_MODEL_STATUS_FALLBACK.missing;
  return template.replaceAll("{command}", command);
}

export function applyTitleModelStatus(root, row, options = {}) {
  const status = root.querySelector("[data-title-model-status]");
  if (!status) {
    return;
  }
  status.textContent = formatTitleModelStatus(row, options);
}

export function parseTitleEnginePhase(raw) {
  const phase = String(raw ?? "").trim();
  return TITLE_ENGINE_PHASES.includes(phase) ? phase : "";
}

export function titleEngineBusy(phase) {
  return phase === "loading" || phase === "hashing";
}

const titleModelPresence = new WeakMap();

function rememberTitleModelPresence(root, rows) {
  if (!root || !Array.isArray(rows)) {
    return;
  }
  const presence = new Map();
  for (const row of rows) {
    const id = parseTitleModelId(row?.id);
    if (id) {
      presence.set(id, row.present === true);
    }
  }
  titleModelPresence.set(root, presence);
}

function rowWithKnownPresence(root, row) {
  if (!row || typeof row.present === "boolean") {
    return row;
  }
  const id = parseTitleModelId(row.id);
  const presence = root ? titleModelPresence.get(root) : undefined;
  if (!id || !presence || !presence.has(id)) {
    return row;
  }
  return { ...row, present: presence.get(id) };
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

export function bundledWeightsAbsent(status, row) {
  const id =
    parseTitleModelId(row?.id) || parseTitleModelId(status?.tier) || "";
  if (!isBundledTitleModel(id)) {
    return false;
  }
  const phase = parseTitleEnginePhase(status?.phase);
  if (phase === "missing") {
    return true;
  }
  return row?.present === false && titleEngineBusy(phase);
}

export function titleEngineProgress(status) {
  const phase = parseTitleEnginePhase(status?.phase);
  if (!titleEngineBusy(phase)) {
    return {
      visible: false,
      determinate: false,
      percent: null,
      value: 0,
      max: 1,
    };
  }
  const total = finiteByteCount(status?.bytesTotal ?? status?.bytes_total);
  const read = finiteByteCount(status?.bytesRead ?? status?.bytes_read);
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

function formatProgressPercent(percent) {
  const template =
    catalogMessage("settings.field.titleModel.progressPercent") ||
    TITLE_MODEL_STATUS_FALLBACK.progressPercent;
  return template.replaceAll("{percent}", String(percent));
}

function notInBuildStatus() {
  return (
    catalogMessage("settings.field.titleModel.notInBuild") ||
    TITLE_MODEL_STATUS_FALLBACK.notInBuild
  );
}

function syncTitleEngineProgress(root, status, absent) {
  const wrap = root.querySelector("[data-title-model-progress]");
  const meter = root.querySelector("#title-model-progress");
  const percentNode = root.querySelector("[data-title-model-progress-value]");
  const view = absent
    ? { visible: false, determinate: false, percent: null, value: 0, max: 1 }
    : titleEngineProgress(status);
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
      meter.setAttribute("aria-labelledby", "title-model-status");
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
      "title-model-status title-model-progress-value",
    );
  }
  if (percentNode) {
    percentNode.hidden = false;
    percentNode.textContent = formatProgressPercent(view.percent);
  }
}

function titleEngineName(id) {
  const key = TITLE_ENGINE_NAME_KEYS[id];
  if (!key) {
    return TITLE_ENGINE_NAME_FALLBACK[id] || id;
  }
  return catalogMessage(key) || TITLE_ENGINE_NAME_FALLBACK[id] || id;
}

function formatLifecycleTemplate(key, fallbackKey, engine) {
  const template =
    catalogMessage(key) || TITLE_MODEL_STATUS_FALLBACK[fallbackKey];
  return template.replaceAll("{engine}", engine);
}

export function formatTitleEngineLifecycle(status, row, options = {}) {
  const selected = parseTitleModelId(options.selected);
  const phase = parseTitleEnginePhase(status?.phase);
  const id =
    selected ||
    parseTitleModelId(status?.tier) ||
    parseTitleModelId(row?.id) ||
    "extractive";
  if (selected === "ollama" || isHostedTitleModel(selected)) {
    return formatTitleModelStatus({ id: selected, ...row }, options);
  }
  if (selected === "custom" && !titleEngineBusy(phase) && phase !== "failed") {
    return formatTitleModelStatus({ id: selected, ...row }, options);
  }
  const described = { id, ...row };
  if (bundledWeightsAbsent(status, described)) {
    return notInBuildStatus();
  }
  const engine = titleEngineName(id);
  if (phase === "loading") {
    return formatLifecycleTemplate(
      "settings.field.titleModel.loading",
      "loading",
      engine,
    );
  }
  if (phase === "hashing") {
    return formatLifecycleTemplate(
      "settings.field.titleModel.hashing",
      "hashing",
      engine,
    );
  }
  if (phase === "ready") {
    return (
      catalogMessage("settings.field.titleModel.loaded") ||
      TITLE_MODEL_STATUS_FALLBACK.loaded
    );
  }
  if (phase === "failed") {
    const reason = String(status?.reason ?? "");
    if (reason === "missing_weights") {
      return formatTitleModelStatus({ id, present: false, ...row }, options);
    }
    const fallbackKey = `failed.${reason}`;
    return (
      catalogMessage(`settings.field.titleModel.failed.${reason}`) ||
      TITLE_MODEL_STATUS_FALLBACK[fallbackKey] ||
      catalogMessage("settings.field.titleModel.failed.unreadable") ||
      TITLE_MODEL_STATUS_FALLBACK["failed.unreadable"]
    );
  }
  if (phase === "missing") {
    return formatTitleModelStatus({ id, present: false, ...row }, options);
  }
  return formatTitleModelStatus({ id, ...row }, options);
}

export function applyTitleEngineLifecycle(root, status, row, options = {}) {
  const node = root.querySelector("[data-title-model-status]");
  const spinner = root.querySelector("[data-title-model-spinner]");
  if (!node) {
    return;
  }
  const absent = bundledWeightsAbsent(status, row);
  node.textContent = formatTitleEngineLifecycle(status, row, options);
  node.setAttribute("aria-live", "polite");
  const busy = !absent && titleEngineBusy(parseTitleEnginePhase(status?.phase));
  if (busy) {
    node.setAttribute("aria-busy", "true");
  } else {
    node.removeAttribute("aria-busy");
  }
  if (spinner) {
    spinner.hidden = !busy;
  }
  syncTitleEngineProgress(root, status, absent);
}

export function bindTitleEngineStatus(root, listenFn = tauriListen) {
  return listenFn(TITLE_ENGINE_STATUS_EVENT, (event) => {
    const payload = event?.payload ?? event;
    bumpTitleEngineRevision(root, parseTitleEnginePhase(payload?.phase));
    const id = parseTitleModelId(payload?.tier) || "extractive";
    applyTitleEngineLifecycle(
      root,
      payload,
      rowWithKnownPresence(root, {
        id,
        vendorCommand: TITLE_MODEL_VENDOR[id] || "",
      }),
    );
  });
}

export async function refreshTitleModelStatus(root, invokeFn, settings) {
  const revision = titleEngineRevision(root);
  const selected = selectedTitleModelId(root, settings);
  const id = selected || "extractive";
  let rows = null;
  try {
    const listed = await invokeFn("list_title_models");
    rows = Array.isArray(listed) ? listed : null;
  } catch {
    rows = null;
  }
  rememberTitleModelPresence(root, rows);
  let engine = null;
  try {
    engine = await invokeFn("title_engine_status");
  } catch {
    engine = null;
  }
  const customName = String(settings?.general?.titleCustomName ?? "").trim();
  const row = rows?.find((item) => item?.id === id) ?? {
    id,
    present:
      rows === null
        ? undefined
        : id === "custom"
          ? customName.length > 0
          : false,
    displayName: customName,
  };
  const options = {
    selected: id,
    unavailable: rows === null && isBundledTitleModel(id),
    unconfirmed: isHostedTitleModel(id)
      ? !settings?.general?.titleHostedConfirmed
      : false,
    host: String(
      root.querySelector("[data-title-hosted-disclosure]")?.dataset
        ?.titleHostedHost ?? "",
    ).trim(),
  };
  const incoming = parseTitleEnginePhase(engine?.phase);
  if (revision !== titleEngineRevision(root)) {
    const liveBusy = titleEngineBusy(notedTitleEnginePhase(root));
    if (!liveBusy || titleEngineBusy(incoming) || !incoming) {
      return rows;
    }
  }
  if (engine && incoming && isBundledTitleModel(id)) {
    rememberTitleEnginePhase(root, incoming);
    applyTitleEngineLifecycle(root, engine, row, options);
  } else if (engine && incoming && id === "extractive") {
    rememberTitleEnginePhase(root, incoming);
    applyTitleEngineLifecycle(root, engine, row, options);
  } else {
    applyTitleModelStatus(root, row, options);
  }
  return rows;
}

export function syncTitleEnginePanels(root, settings = {}) {
  const id = selectedTitleModelId(root, settings);
  const showCustom = id === "custom";
  const showOllama = id === "ollama";
  const showHosted = isHostedTitleModel(id);
  const usingIntegration = isIntegrationTitleModel(id);
  const titles = root.querySelector('[data-settings-group="titles"]');
  if (titles?.dataset) {
    titles.dataset.titleEngineSource = usingIntegration
      ? "integration"
      : "local";
  }
  const custom = root.querySelector('[data-title-engine-panel="custom"]');
  const ollama = root.querySelector('[data-title-engine-panel="ollama"]');
  const hosted = root.querySelector('[data-title-engine-panel="hosted"]');
  if (custom) {
    custom.hidden = !showCustom;
  }
  if (ollama) {
    ollama.hidden = !showOllama;
  }
  if (hosted) {
    hosted.hidden = !showHosted;
  }
  const helpOllama = root.querySelector("#title-model-help-ollama");
  const helpHosted = root.querySelector("#title-model-help-hosted");
  const helpIntegrations = root.querySelector("#title-model-help-integrations");
  const helpLocal = root.querySelector("#title-model-help");
  const helpExtractive = root.querySelector("#title-model-help-extractive");
  const helpSize = root.querySelector("#title-model-help-size");
  if (helpOllama) {
    helpOllama.hidden = !showOllama;
  }
  if (helpHosted) {
    helpHosted.hidden = !showHosted;
  }
  if (helpIntegrations) {
    helpIntegrations.hidden = showOllama || showHosted;
  }
  if (helpLocal) {
    helpLocal.hidden = showHosted;
  }
  if (helpExtractive) {
    helpExtractive.hidden = id !== "extractive";
  }
  if (helpSize) {
    helpSize.hidden = !(isBundledTitleModel(id) || id === "custom");
  }
  const file = root.querySelector("[data-title-custom-file]");
  if (file) {
    const name = String(settings?.general?.titleCustomName ?? "").trim();
    const bytes = Number(settings?.general?.titleCustomBytes ?? 0);
    if (name && !name.includes("/")) {
      const template =
        catalogMessage("settings.field.titleModel.imported") ||
        TITLE_MODEL_STATUS_FALLBACK.imported;
      file.textContent = template
        .replaceAll("{name}", name)
        .replaceAll("{size}", formatTitleFileSize(bytes));
    } else {
      file.textContent =
        catalogMessage("settings.field.titleModel.custom.empty") ||
        TITLE_MODEL_STATUS_FALLBACK.customEmpty;
    }
  }
  const customOption = root.querySelector(
    '#title-model option[value="custom"]',
  );
  if (customOption) {
    customOption.textContent = formatCustomTitleOption(
      settings?.general?.titleCustomBytes,
    );
  }
}

export function fillOllamaTitleModels(select, models, saved) {
  if (!select) {
    return;
  }
  const names = [];
  const seen = new Set();
  for (const raw of [saved, ...(Array.isArray(models) ? models : [])]) {
    const name = String(raw ?? "").trim();
    if (!name || name.includes("/") || seen.has(name)) {
      continue;
    }
    seen.add(name);
    names.push(name);
  }
  const doc = select.ownerDocument;
  if (!doc?.createElement) {
    select.value = saved || "";
    return;
  }
  select.replaceChildren();
  for (const name of names) {
    const option = doc.createElement("option");
    option.value = name;
    option.textContent = name;
    select.append(option);
  }
  if (saved && seen.has(saved)) {
    select.value = saved;
  }
}

export async function refreshHostedDisclosure(root, invokeFn, settings) {
  const id = selectedTitleModelId(root, settings);
  const node = root.querySelector("[data-title-hosted-disclosure]");
  if (!node || !isHostedTitleModel(id)) {
    return "";
  }
  const customBase = String(
    root.querySelector("#title-hosted-base")?.value ??
      settings?.general?.titleHostedBase ??
      "",
  ).trim();
  try {
    const dto = await invokeFn("hosted_title_disclosure", {
      provider: id,
      customBase,
    });
    const host = String(dto?.host ?? "").trim();
    if (!host || host.includes("/")) {
      throw new Error("blocked_host");
    }
    node.dataset.titleHostedHost = host;
    const template =
      catalogMessage("settings.field.titleModel.disclosure") ||
      TITLE_MODEL_STATUS_FALLBACK.disclosure;
    node.textContent = template.replaceAll("{host}", host);
    return host;
  } catch {
    node.dataset.titleHostedHost = "";
    node.textContent =
      catalogMessage("settings.field.titleModel.hosted.blocked") ||
      TITLE_MODEL_STATUS_FALLBACK.hostedBlocked;
    return "";
  }
}

export function switcherLocale(tag) {
  if (SWITCHER_LOCALES.includes(tag)) {
    return tag;
  }
  return "en";
}

export function settingsSearchNeedle(raw) {
  return String(raw ?? "")
    .trim()
    .toLocaleLowerCase();
}

export function settingsSearchMatches(text, needle) {
  if (!needle) {
    return true;
  }
  return String(text ?? "")
    .toLocaleLowerCase()
    .includes(needle);
}

export function settingsUnitHaystack(unit) {
  const label = unit.querySelector?.("label")?.textContent ?? "";
  const help = [
    ...(unit.querySelectorAll?.(
      ".field-help, [data-excluded-help], [data-excluded-howto], [data-title-model-help], [data-setting-info]",
    ) ?? []),
  ]
    .map((node) => node.textContent ?? "")
    .join(" ");
  const chips = [...(unit.querySelectorAll?.("[data-app-name]") ?? [])]
    .map((node) => node.getAttribute?.("data-app-name") ?? "")
    .join(" ");
  const options = [...(unit.querySelectorAll?.("option") ?? [])]
    .map((option) => `${option.textContent ?? ""} ${option.value ?? ""}`)
    .join(" ");
  const control = unit.querySelector?.("input, select, textarea");
  const controlText = control
    ? `${control.value ?? ""} ${control.getAttribute?.("name") ?? ""}`
    : "";
  if (label || help || chips || options || controlText.trim()) {
    return `${label} ${help} ${chips} ${options} ${controlText}`;
  }
  return unit.textContent ?? "";
}

export function isSafeDisplayName(name) {
  const value = String(name ?? "").trim();
  return (
    value.length > 0 &&
    !value.includes("/") &&
    !value.includes("\\") &&
    !value.includes("\0") &&
    !value.includes("..")
  );
}

export function pickerFailureCode(error) {
  const text =
    typeof error === "string" ? error : String(error?.message ?? error ?? "");
  if (text.includes("picker_cancelled")) {
    return "picker_cancelled";
  }
  if (text.includes("picker_unavailable")) {
    return "picker_unavailable";
  }
  return "invalid_app";
}

export function addExcludedApp(host, app) {
  if (!isSafeBundleId(app?.bundleId)) {
    return false;
  }
  const bundleId = app.bundleId.trim();
  const rawName = String(app.name ?? "").trim();
  const picked = {
    bundleId,
    name: isSafeDisplayName(rawName) ? rawName : bundleId,
  };
  const catalog = [...(host?._installedApps ?? []), picked];
  if (host) {
    host._installedApps = catalog;
  }
  writeExcludedApps(host, [
    ...resolveExcludedApps(readExcludedBundleIds(host), catalog),
    picked,
  ]);
  return true;
}

export async function pickExcludedApp(host, invokeFn) {
  try {
    const app = await invokeFn("pick_installed_app");
    if (!addExcludedApp(host, app)) {
      return "invalid_app";
    }
    return "picked";
  } catch (error) {
    return pickerFailureCode(error);
  }
}

export function isSafeBundleId(id) {
  const value = String(id ?? "").trim();
  return (
    value.length > 0 &&
    value.length <= 256 &&
    !value.includes("/") &&
    !value.includes("\\") &&
    !value.includes("\0") &&
    !value.includes("..")
  );
}

export function filterInstalledApps(apps, query, selectedIds) {
  const needle = settingsSearchNeedle(query);
  const selected = new Set(
    (selectedIds ?? [])
      .filter(isSafeBundleId)
      .map((id) => id.trim().toLocaleLowerCase()),
  );
  return (Array.isArray(apps) ? apps : []).filter((app) => {
    if (!isSafeBundleId(app?.bundleId)) {
      return false;
    }
    if (selected.has(app.bundleId.trim().toLocaleLowerCase())) {
      return false;
    }
    return settingsSearchMatches(`${app.name ?? ""} ${app.bundleId}`, needle);
  });
}

export function resolveExcludedApps(ids, catalog) {
  const byId = new Map();
  for (const app of Array.isArray(catalog) ? catalog : []) {
    if (!isSafeBundleId(app?.bundleId)) {
      continue;
    }
    byId.set(app.bundleId.trim().toLocaleLowerCase(), {
      bundleId: app.bundleId.trim(),
      name: String(app.name ?? "").trim() || app.bundleId.trim(),
    });
  }
  const out = [];
  const seen = new Set();
  for (const raw of ids ?? []) {
    if (!isSafeBundleId(raw)) {
      continue;
    }
    const key = raw.trim().toLocaleLowerCase();
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    out.push(
      byId.get(key) ?? {
        bundleId: raw.trim(),
        name: raw.trim(),
      },
    );
  }
  return out;
}

export function readExcludedBundleIds(host) {
  const raw = host?.dataset?.excludedIds;
  if (typeof raw === "string") {
    return raw
      .split("\n")
      .map((value) => value.trim())
      .filter(isSafeBundleId);
  }
  return [...(host?.querySelectorAll?.("[data-excluded-app]") ?? [])]
    .map((node) => node.getAttribute?.("data-bundle-id"))
    .filter(isSafeBundleId);
}

export function writeExcludedApps(host, apps) {
  const selected = resolveExcludedApps(
    (apps ?? []).map((app) => app.bundleId),
    apps,
  );
  if (host?.dataset) {
    host.dataset.excludedIds = selected.map((app) => app.bundleId).join("\n");
  }
  renderExcludedChips(host, selected);
  return selected;
}

export function renderExcludedChips(host, apps) {
  const list = host?.querySelector?.("#excluded-apps-selected");
  if (!list?.replaceChildren) {
    return;
  }
  list.replaceChildren();
  const removeLabel =
    catalogMessage("settings.field.excludedBundleIds.remove") || "Remove";
  for (const app of apps) {
    const item = globalThis.document?.createElement?.("li");
    if (!item) {
      continue;
    }
    item.className = "app-chip";
    item.dataset.excludedApp = "";
    item.dataset.bundleId = app.bundleId;
    item.dataset.appName = app.name;
    const icon = globalThis.document.createElement("img");
    icon.alt = "";
    icon.className = "app-picker-option-icon";
    icon.width = 16;
    icon.height = 16;
    const name = globalThis.document.createElement("span");
    name.className = "app-chip-name";
    name.textContent = app.name;
    const remove = globalThis.document.createElement("button");
    remove.type = "button";
    remove.className = "btn-icon";
    remove.dataset.removeExcluded = app.bundleId;
    remove.setAttribute(
      "data-i18n-aria-label",
      "settings.field.excludedBundleIds.remove",
    );
    remove.setAttribute("aria-label", removeLabel);
    const mark = globalThis.document.createElementNS(
      "http://www.w3.org/2000/svg",
      "svg",
    );
    mark.setAttribute("viewBox", "0 0 12 12");
    mark.setAttribute("aria-hidden", "true");
    mark.setAttribute("focusable", "false");
    const path = globalThis.document.createElementNS(
      "http://www.w3.org/2000/svg",
      "path",
    );
    path.setAttribute(
      "d",
      "M2.1 1.4 1.4 2.1 5.3 6 1.4 9.9l.7.7L6 6.7l3.9 3.9.7-.7L6.7 6l3.9-3.9-.7-.7L6 5.3z",
    );
    path.setAttribute("fill", "currentColor");
    mark.append(path);
    remove.append(mark);
    item.append(icon, name, remove);
    list.append(item);
  }
}

export function applySettingsSearch(root, rawQuery) {
  const needle = settingsSearchNeedle(rawQuery);
  const searching = needle.length > 0;

  for (const unit of root.querySelectorAll("[data-settings-unit]")) {
    unit.hidden =
      searching && !settingsSearchMatches(settingsUnitHaystack(unit), needle);
  }

  for (const section of root.querySelectorAll("[data-settings-section]")) {
    const title =
      section.querySelector("[data-settings-title]")?.textContent ?? "";
    const info = [
      ...(section.querySelectorAll?.(
        ":scope > .setting-heading [data-setting-info], legend [data-setting-info]",
      ) ?? []),
    ]
      .map((node) => node.textContent ?? "")
      .join(" ");
    const titleHit = settingsSearchMatches(`${title} ${info}`, needle);
    const units = [...section.querySelectorAll("[data-settings-unit]")];
    if (searching && titleHit) {
      for (const unit of units) {
        unit.hidden = false;
      }
      section.hidden = false;
      continue;
    }
    if (units.length > 0) {
      section.hidden = searching && units.every((unit) => unit.hidden);
      continue;
    }
    section.hidden =
      searching && !settingsSearchMatches(section.textContent ?? "", needle);
  }

  const form = root.querySelector("[data-settings-form]");
  if (form) {
    const groups = [...form.querySelectorAll("[data-settings-section]")];
    form.hidden = searching && groups.every((section) => section.hidden);
  }

  const empty = root.querySelector("[data-settings-search-empty]");
  if (empty) {
    const visible = [...root.querySelectorAll("[data-settings-section]")].some(
      (section) => !section.hidden,
    );
    empty.hidden = !searching || visible;
  }
}

const EXPORT_CATEGORY_FALLBACK = {
  general: "General",
  capture: "Capture",
  panel: "Panel",
  copy: "Copy",
  privacy: "Privacy",
  data: "Data",
  accessibility: "Accessibility",
  shortcuts: "Shortcuts",
  profiles: "Profiles",
};

const EXPORT_SENSITIVE_KEYS = {
  "privacy.excludedBundleIds": "settings.export.key.excludedApps",
  "privacy.appPolicies": "settings.export.key.appPolicies",
  "capture.standardChord": "settings.export.key.captureShortcut",
  "copy.defaultProfileId": "settings.export.key.defaultProfile",
};

const EXPORT_SENSITIVE_FALLBACK = {
  "settings.export.key.excludedApps": "Excluded apps",
  "settings.export.key.appPolicies": "App policies",
  "settings.export.key.captureShortcut": "Capture shortcut",
  "settings.export.key.defaultProfile": "Default profile",
  "settings.export.key.customShortcut": "Custom shortcut",
  "settings.export.key.profileLiterals": "Profile literals",
};

const EXPORT_FAILURE_KEYS = {
  picker_unavailable: "settings.export.unavailable",
  settings_invalid: "settings.export.invalid",
  settings_import_not_json: "settings.export.invalid",
  settings_path_invalid: "settings.export.invalid",
  settings_forbidden: "settings.export.forbidden",
  settings_wrong_format: "settings.export.wrongFormat",
  settings_unknown_version: "settings.export.unknownVersion",
  settings_import_too_large: "settings.export.invalid",
  webview_path_rejected: "settings.export.invalid",
};

export function settingsExportFailureCode(error) {
  const text =
    typeof error === "string" ? error : String(error?.message ?? error ?? "");
  if (!text || text.includes("picker_cancelled")) {
    return "";
  }
  for (const code of Object.keys(EXPORT_FAILURE_KEYS)) {
    if (text.includes(code)) {
      return code;
    }
  }
  return "settings_invalid";
}

export function exportCategoryLabel(id) {
  if (id === "shortcuts") {
    return (
      catalogMessage("settings.shortcuts.title") ||
      EXPORT_CATEGORY_FALLBACK.shortcuts
    );
  }
  if (id === "profiles") {
    return (
      catalogMessage("settings.export.category.profiles") ||
      EXPORT_CATEGORY_FALLBACK.profiles
    );
  }
  return (
    catalogMessage(`settings.group.${id}`) || EXPORT_CATEGORY_FALLBACK[id] || id
  );
}

export function exportSensitiveLabel(key) {
  const mapped = EXPORT_SENSITIVE_KEYS[key];
  if (mapped) {
    return catalogMessage(mapped) || EXPORT_SENSITIVE_FALLBACK[mapped] || key;
  }
  if (String(key).startsWith("shortcuts.")) {
    return (
      catalogMessage("settings.export.key.customShortcut") ||
      EXPORT_SENSITIVE_FALLBACK["settings.export.key.customShortcut"]
    );
  }
  if (String(key).startsWith("profiles.")) {
    return (
      catalogMessage("settings.export.key.profileLiterals") ||
      EXPORT_SENSITIVE_FALLBACK["settings.export.key.profileLiterals"]
    );
  }
  return key;
}

export function renderExportPreview(root, preview) {
  const included = root.querySelector("[data-export-included]");
  const sensitive = root.querySelector("[data-export-sensitive-list]");
  if (included) {
    included.replaceChildren();
    const categories = [
      ...new Set(
        (preview?.includedCategories ?? []).map((id) =>
          exportCategoryLabel(id),
        ),
      ),
    ].filter(Boolean);
    for (const label of categories) {
      const item = globalThis.document.createElement("li");
      item.textContent = label;
      included.append(item);
    }
  }
  if (sensitive) {
    sensitive.replaceChildren();
    const labels = [
      ...new Set(
        (preview?.sensitiveLiteralKeys ?? []).map((key) =>
          exportSensitiveLabel(key),
        ),
      ),
    ].filter(Boolean);
    if (labels.length === 0) {
      const item = globalThis.document.createElement("li");
      item.textContent =
        catalogMessage("settings.export.emptySensitive") ||
        "No extra user-entered literals in this file.";
      sensitive.append(item);
      return;
    }
    for (const label of labels) {
      const item = globalThis.document.createElement("li");
      item.textContent = label;
      sensitive.append(item);
    }
  }
}

function showExportStatus(root, key) {
  const status = root.querySelector("[data-export-status]");
  if (!status) {
    return;
  }
  if (!key) {
    status.textContent = "";
    return;
  }
  status.textContent =
    catalogMessage(key) ||
    (key === "settings.export.done"
      ? "Exported"
      : key === "settings.import.done"
        ? "Imported"
        : "That settings file is not valid.");
}

const LOGIN_ITEM_STATUS_KEYS = Object.freeze({
  enabled: "settings.field.launchAtLogin.status.enabled",
  not_registered: "settings.field.launchAtLogin.status.notRegistered",
  requires_approval: "settings.field.launchAtLogin.status.requiresApproval",
  unavailable: "settings.field.launchAtLogin.status.unavailable",
});

const LOGIN_ITEM_STATUS_FALLBACK = Object.freeze({
  enabled: "Registered to open at login.",
  not_registered: "Not registered to open at login.",
  requires_approval: "Allow Bronze in Login Items to finish.",
  unavailable: "This debug build cannot register as a login item.",
});

const VERSION_SOURCE_KEYS = {
  debug: "settings.field.version.source.debug",
  homebrew: "settings.field.version.source.homebrew",
  direct: "settings.field.version.source.direct",
  unknown: "settings.field.version.source.unknown",
};

const VERSION_SOURCE_FALLBACK = {
  debug: "Debug build",
  homebrew: "Installed with Homebrew",
  direct: "Installed from a package",
  unknown: "Install source unknown",
};

export function parseInstallSource(raw) {
  if (
    raw === "debug" ||
    raw === "homebrew" ||
    raw === "direct" ||
    raw === "unknown"
  ) {
    return raw;
  }
  return "unknown";
}

export function parseUpdateAction(raw) {
  if (
    raw === "open-release" ||
    raw === "brew-upgrade" ||
    raw === "debug" ||
    raw === "none"
  ) {
    return raw;
  }
  return "none";
}

function formatAvailableVersion(version) {
  return (
    catalogMessage("settings.field.version.available") ||
    "Version {version} is available."
  ).replaceAll("{version}", version || "—");
}

export function applyAppVersionInfo(root, info) {
  const version = String(info?.version ?? "").trim();
  const number = root.querySelector("[data-app-version-number]");
  const sourceEl = root.querySelector("[data-app-version-source]");
  if (number) {
    number.textContent =
      version ||
      catalogMessage("settings.field.version.unknown") ||
      "Version unavailable";
  }
  if (sourceEl) {
    const source = parseInstallSource(info?.installSource);
    sourceEl.hidden = false;
    sourceEl.dataset.appVersionSource = source;
    sourceEl.textContent =
      catalogMessage(VERSION_SOURCE_KEYS[source]) ||
      VERSION_SOURCE_FALLBACK[source];
  }
}

export async function refreshAppVersion(root, invokeFn) {
  try {
    const info = await invokeFn("app_version_info");
    applyAppVersionInfo(root, info);
    return info;
  } catch {
    applyAppVersionInfo(root, { version: "", installSource: "unknown" });
    return null;
  }
}

export function renderUpdateNotes(container, notes) {
  if (!container?.replaceChildren) {
    return;
  }
  const items = (Array.isArray(notes) ? notes : [])
    .map((note) => String(note ?? "").trim())
    .filter(Boolean);
  if (!items.length) {
    const empty = container.ownerDocument.createElement("p");
    empty.className = "muted";
    empty.textContent =
      catalogMessage("settings.field.version.notesEmpty") ||
      "Release notes were not published for this version.";
    container.replaceChildren(empty);
    return;
  }
  const list = container.ownerDocument.createElement("ul");
  for (const note of items) {
    const item = container.ownerDocument.createElement("li");
    item.textContent = note.replace(/^•\s*/, "");
    list.append(item);
  }
  container.replaceChildren(list);
}

export function applyUpdateCheck(root, check) {
  const status = root.querySelector("[data-app-version-status]");
  const dialog = root.querySelector("#update-sheet");
  const actionBtn = root.querySelector("[data-update-action]");
  const availableLine = root.querySelector("[data-update-available]");
  const notes = root.querySelector("[data-update-notes]");
  const debugLine = root.querySelector("[data-update-debug]");
  const commandEl = root.querySelector("[data-update-command]");
  const latest = String(check?.latestVersion ?? "").trim();
  const available = Boolean(check?.available);
  if (status) {
    status.hidden = false;
    status.textContent = available
      ? formatAvailableVersion(latest)
      : catalogMessage("settings.field.version.current") ||
        "This is the latest version.";
  }
  if (!available) {
    return check;
  }
  if (availableLine) {
    availableLine.textContent = formatAvailableVersion(latest);
  }
  renderUpdateNotes(notes, check?.notes);
  const action = parseUpdateAction(check?.action);
  if (actionBtn) {
    actionBtn.hidden = action === "none" || action === "debug";
    actionBtn.dataset.updateAction = action;
    actionBtn.dataset.releaseUrl = String(check?.releaseUrl ?? "");
    actionBtn.dataset.actionCommand = String(check?.actionCommand ?? "");
    if (action === "brew-upgrade") {
      actionBtn.className = "btn-primary";
      actionBtn.textContent =
        catalogMessage("settings.field.version.brew") ||
        "Copy Homebrew command";
    } else if (action === "open-release") {
      actionBtn.className = "btn-primary";
      actionBtn.textContent =
        catalogMessage("settings.field.version.openRelease") || "Open release";
    } else {
      actionBtn.className = "btn-ghost";
    }
  }
  if (debugLine) {
    debugLine.hidden = action !== "debug";
  }
  if (commandEl) {
    commandEl.hidden = true;
    commandEl.textContent = "";
  }
  if (typeof dialog?.showModal === "function") {
    dialog.showModal();
  }
  return check;
}

export async function checkForAppUpdate(root, invokeFn) {
  const status = root.querySelector("[data-app-version-status]");
  const button = root.querySelector("[data-check-update]");
  if (status) {
    status.hidden = false;
    status.textContent =
      catalogMessage("settings.field.version.checking") ||
      "Checking for updates";
  }
  if (button) {
    button.disabled = true;
  }
  try {
    const check = await invokeFn("check_for_update");
    return applyUpdateCheck(root, check);
  } catch {
    if (status) {
      status.textContent =
        catalogMessage("settings.field.version.failed") ||
        "The update check could not reach GitHub.";
    }
    return null;
  } finally {
    if (button) {
      button.disabled = false;
    }
  }
}

export function bindAppVersion(root, invokeFn) {
  root.querySelector("[data-check-update]")?.addEventListener("click", () => {
    void checkForAppUpdate(root, invokeFn);
  });
  root
    .querySelector("[data-update-action]")
    ?.addEventListener("click", async () => {
      const actionBtn = root.querySelector("[data-update-action]");
      const action = parseUpdateAction(actionBtn?.dataset?.updateAction);
      const url = String(actionBtn?.dataset?.releaseUrl ?? "");
      const command = String(actionBtn?.dataset?.actionCommand ?? "");
      if (action === "open-release") {
        if (
          !url.startsWith("https:") ||
          !url.includes("//github.com/NxT-Solutions/Bronze/releases")
        ) {
          return;
        }
        await invokeFn("open_release_page", { url });
        return;
      }
      if (action !== "brew-upgrade" || !command) {
        return;
      }
      try {
        await navigator.clipboard.writeText(command);
        actionBtn.textContent =
          catalogMessage("settings.field.version.brewCopied") || "Copied";
      } catch {
        const commandEl = root.querySelector("[data-update-command]");
        if (commandEl) {
          commandEl.hidden = false;
          commandEl.textContent = command;
        }
      }
    });
}

export function parseLoginItemStatus(raw) {
  if (raw && LOGIN_ITEM_STATUS_KEYS[raw]) {
    return raw;
  }
  return "unavailable";
}

export async function refreshLoginItemStatus(root, invokeFn) {
  const el = root.querySelector("[data-login-item-status]");
  if (!el) {
    return "unavailable";
  }
  let raw = "unavailable";
  try {
    raw = await invokeFn("login_item_status");
  } catch {
    raw = "unavailable";
  }
  const status = parseLoginItemStatus(raw);
  const key = LOGIN_ITEM_STATUS_KEYS[status];
  el.hidden = false;
  el.dataset.loginItemStatus = status;
  el.textContent = catalogMessage(key) || LOGIN_ITEM_STATUS_FALLBACK[status];
  return status;
}

export function applySettingsForm(root, settings) {
  const schedule = root.querySelector("#backup-schedule");
  const locale = root.querySelector("#ui-locale");
  const titleModel = root.querySelector("#title-model");
  const host = root.querySelector("#excluded-apps");
  const launchAtLogin = root.querySelector("#launch-at-login");
  if (launchAtLogin) {
    launchAtLogin.checked = Boolean(settings?.general?.launchAtLogin);
  }
  if (schedule && settings?.data?.backupSchedule) {
    schedule.value = settings.data.backupSchedule;
  }
  if (host) {
    writeExcludedApps(
      host,
      resolveExcludedApps(
        settings?.privacy?.excludedBundleIds ?? [],
        host._installedApps ?? [],
      ),
    );
  }
  if (locale) {
    locale.value = switcherLocale(settings?.general?.locale);
  }
  const titleId = parseTitleModelId(settings?.general?.titleModel);
  const titleIntegration = root.querySelector("#title-integration");
  if (titleModel && isLocalTitleModel(titleId)) {
    titleModel.value = titleId;
  }
  if (titleIntegration) {
    titleIntegration.value = isIntegrationTitleModel(titleId)
      ? titleId
      : "none";
  }
  const ollama = root.querySelector("#title-ollama-model");
  const savedOllama = String(settings?.general?.titleOllamaModel ?? "").trim();
  if (ollama && savedOllama && !savedOllama.includes("/")) {
    if (!ollama.options?.length) {
      fillOllamaTitleModels(ollama, [], savedOllama);
    } else {
      ollama.value = savedOllama;
    }
  }
  const hostedBase = root.querySelector("#title-hosted-base");
  if (hostedBase) {
    hostedBase.value = String(settings?.general?.titleHostedBase ?? "");
  }
  const hostedConfirmed = root.querySelector("#title-hosted-confirmed");
  if (hostedConfirmed) {
    hostedConfirmed.checked = Boolean(settings?.general?.titleHostedConfirmed);
  }
  const hostedKey = root.querySelector("#title-hosted-key");
  if (hostedKey) {
    hostedKey.value = "";
  }
  syncTitleEnginePanels(root, settings);
  const reduceMotion = root.querySelector("#reduce-motion");
  if (reduceMotion) {
    reduceMotion.value = parseReduceMotion(
      settings?.general?.reduceMotion ?? settings?.accessibility?.motion,
    );
  }
  const queueSort = root.querySelector("#queue-sort");
  if (queueSort) {
    queueSort.value = parseQueueSort(settings?.copy?.queueSort);
  }
}

export function patchSettingsFromForm(settings, root) {
  const next = structuredClone(settings);
  if (!next.general) {
    next.general = {};
  }
  const schedule = root.querySelector("#backup-schedule")?.value;
  const locale = root.querySelector("#ui-locale")?.value;
  const titleModel = selectedTitleModelId(root, settings);
  if (schedule === "daily" || schedule === "weekly") {
    next.data.backupSchedule = schedule;
  }
  next.privacy.excludedBundleIds = readExcludedBundleIds(
    root.querySelector("#excluded-apps"),
  );
  if (SWITCHER_LOCALES.includes(locale)) {
    next.general.locale = locale;
  }
  if (titleModel) {
    next.general.titleModel = titleModel;
  }
  const ollamaModel = String(
    root.querySelector("#title-ollama-model")?.value ??
      settings?.general?.titleOllamaModel ??
      "",
  ).trim();
  if (!ollamaModel.includes("/")) {
    next.general.titleOllamaModel = ollamaModel;
  }
  next.general.titleHostedBase = String(
    root.querySelector("#title-hosted-base")?.value ?? "",
  ).trim();
  next.general.titleHostedConfirmed = Boolean(
    root.querySelector("#title-hosted-confirmed")?.checked,
  );
  if (settings?.general?.titleCustomId) {
    next.general.titleCustomId = settings.general.titleCustomId;
    next.general.titleCustomName = settings.general.titleCustomName;
    next.general.titleCustomBytes = settings.general.titleCustomBytes;
  }
  const reduceEl = root.querySelector("#reduce-motion")?.value;
  if (reduceEl === "system" || reduceEl === "on" || reduceEl === "off") {
    next.general.reduceMotion = reduceEl;
    if (!next.accessibility) {
      next.accessibility = {};
    }
    next.accessibility.motion = reduceEl;
  }
  const launchAtLogin = root.querySelector("#launch-at-login");
  if (launchAtLogin && typeof launchAtLogin.checked === "boolean") {
    next.general.launchAtLogin = launchAtLogin.checked;
  }
  const queueSort = root.querySelector("#queue-sort")?.value;
  if (queueSort === "newest" || queueSort === "oldest") {
    next.copy = { ...(next.copy ?? {}), queueSort };
  }
  return next;
}

async function publishQueueSort(before, after) {
  const change = queueSortChange(before, after);
  if (change) {
    await emitQueueSortChanged(change);
  }
}

async function applySavedLocale(root, settings, invokeFn) {
  applySettingsForm(root, settings);
  applyMotionFromSettings(root, settings);
  try {
    const catalog = await invokeFn("ui_catalog");
    applyHandTestLocale(root, settings?.general?.locale, catalog);
  } catch {
    applyHandTestLocale(root, switcherLocale(settings?.general?.locale));
  }
  syncTitleEnginePanels(root, settings);
  await refreshTitleModelStatus(root, invokeFn, settings);
  await refreshLoginItemStatus(root, invokeFn);
  await refreshAppVersion(root, invokeFn);
}

export function bindSearchClear(root = document) {
  for (const wrap of root.querySelectorAll(".chrome-search")) {
    const input = wrap.querySelector("input[type='search']");
    const clear = wrap.querySelector("[data-search-clear]");
    if (!input || !clear) {
      continue;
    }
    const sync = () => {
      clear.hidden = String(input.value ?? "").length === 0;
    };
    input.addEventListener("input", sync);
    clear.addEventListener("click", () => {
      input.value = "";
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.focus();
    });
    sync();
  }
}

export function bindSettingInfo(root) {
  const infos = [...(root.querySelectorAll?.("details.setting-info") ?? [])];
  if (infos.length === 0) {
    return;
  }
  const dismissed = new WeakSet();
  const closeAll = (except) => {
    for (const el of infos) {
      if (el !== except && el.open) {
        el.open = false;
      }
    }
  };
  const show = (el) => {
    if (dismissed.has(el)) {
      return;
    }
    el.open = true;
    closeAll(el);
  };
  const hide = (el) => {
    el.open = false;
  };
  const focusInside = (el) => {
    const active = el.ownerDocument?.activeElement;
    return Boolean(active && el.contains?.(active));
  };
  for (const el of infos) {
    el.addEventListener("pointerenter", () => {
      show(el);
    });
    el.addEventListener("pointerleave", () => {
      dismissed.delete(el);
      if (!focusInside(el)) {
        hide(el);
      }
    });
    el.addEventListener("focusin", () => {
      show(el);
    });
    el.addEventListener("focusout", (event) => {
      const next = event.relatedTarget;
      if (next && el.contains?.(next)) {
        return;
      }
      if (el.matches?.(":hover")) {
        return;
      }
      dismissed.delete(el);
      hide(el);
    });
    el.querySelector("summary")?.addEventListener("click", (event) => {
      event.preventDefault();
    });
  }
  root.addEventListener("pointerdown", (event) => {
    if (event.target?.closest?.("details.setting-info")) {
      return;
    }
    closeAll();
  });
  root.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") {
      return;
    }
    if (root.querySelector?.("[data-recording='true']")) {
      return;
    }
    const open = infos.find((el) => el.open);
    if (!open) {
      return;
    }
    dismissed.add(open);
    hide(open);
    open.querySelector("summary")?.focus?.();
    event.preventDefault();
  });
}

export async function bindSettingsLive(
  root = document,
  invokeFn = tauriInvoke,
) {
  bindSettingInfo(root);
  bindSearchClear(root);
  const search = root.querySelector("#settings-search");
  const filterSettings = () => {
    const query = search?.value ?? "";
    search
      ?.closest?.(".chrome-search")
      ?.classList.toggle("is-filled", query.trim().length > 0);
    applySettingsSearch(root, query);
  };
  search?.addEventListener("input", filterSettings);
  filterSettings();

  const form = root.querySelector("#settings form");
  if (!form) {
    return;
  }
  let settings;
  try {
    settings = await invokeFn("load_settings_v1");
  } catch {
    return;
  }

  const host = root.querySelector("#excluded-apps");
  const unavailable = root.querySelector("[data-excluded-apps-unavailable]");
  const appSearch = root.querySelector("#excluded-apps-search");
  let installedApps = [];
  let appsUnavailable = false;
  try {
    const listed = await invokeFn("list_installed_apps");
    if (!Array.isArray(listed)) {
      appsUnavailable = true;
    } else {
      installedApps = listed.filter((app) => isSafeBundleId(app?.bundleId));
    }
  } catch {
    appsUnavailable = true;
  }
  if (host) {
    host._installedApps = installedApps;
    host._appsUnavailable = appsUnavailable;
  }
  if (unavailable) {
    unavailable.hidden = !appsUnavailable;
  }
  if (appSearch) {
    appSearch.disabled = appsUnavailable;
  }
  await applySavedLocale(root, settings, invokeFn);
  bindAppVersion(root, invokeFn);
  bindTitleEngineStatus(root);
  await refreshExcludedIcons(root, invokeFn);
  bindExcludedPicker(root, invokeFn, () => persist());
  const refreshShortcuts = await bindShortcutRegistry(root, invokeFn);

  let ollamaListed = false;
  async function listOllamaIfNeeded() {
    const id = selectedTitleModelId(root, settings);
    if (id !== "ollama" || ollamaListed) {
      return;
    }
    ollamaListed = true;
    const select = root.querySelector("#title-ollama-model");
    try {
      const models = await invokeFn("list_ollama_title_models");
      fillOllamaTitleModels(
        select,
        models,
        settings?.general?.titleOllamaModel,
      );
    } catch {
      fillOllamaTitleModels(select, [], settings?.general?.titleOllamaModel);
    }
  }

  async function refreshEngineExtras() {
    const id = selectedTitleModelId(root, settings);
    if (id === "ollama") {
      await listOllamaIfNeeded();
    }
    if (isHostedTitleModel(id)) {
      await refreshHostedDisclosure(root, invokeFn, settings);
    }
  }

  async function persist() {
    const before = settings?.general?.locale;
    const beforeSort = settings?.copy?.queueSort;
    settings = await invokeFn("save_settings_v1", {
      settings: patchSettingsFromForm(settings, root),
    });
    const keyInput = root.querySelector("#title-hosted-key");
    const key = String(keyInput?.value ?? "").trim();
    const provider = selectedTitleModelId(root, settings);
    if (key && isHostedTitleModel(provider)) {
      await invokeFn("set_hosted_title_key", { provider, key });
      keyInput.value = "";
    }
    await applySavedLocale(root, settings, invokeFn);
    await refreshEngineExtras();
    await refreshExcludedIcons(root, invokeFn);
    await refreshShortcuts?.();
    await refreshExportPreview();
    await emitMotionChanged({
      reduceMotion: settings?.general?.reduceMotion,
    });
    if (settings?.general?.locale !== before) {
      await emitUiLocaleChanged({ locale: settings.general.locale });
    }
    await publishQueueSort(beforeSort, settings?.copy?.queueSort);
  }

  async function refreshExportPreview() {
    try {
      const preview = await invokeFn("preview_settings_export");
      renderExportPreview(root, preview);
    } catch {
      renderExportPreview(root, {
        includedCategories: [],
        sensitiveLiteralKeys: [],
      });
    }
  }

  const exportButton = root.querySelector("[data-export-settings]");
  exportButton?.addEventListener("click", () => {
    runBusy(exportButton, async () => {
      try {
        await invokeFn("export_settings_file", { requestedPath: null });
        showExportStatus(root, "settings.export.done");
        await refreshExportPreview();
      } catch (error) {
        const code = settingsExportFailureCode(error);
        showExportStatus(root, code ? EXPORT_FAILURE_KEYS[code] : "");
      }
    });
  });

  const importButton = root.querySelector("[data-import-settings]");
  importButton?.addEventListener("click", () => {
    runBusy(importButton, async () => {
      try {
        const before = settings?.general?.locale;
        const beforeSort = settings?.copy?.queueSort;
        settings = await invokeFn("import_settings_file", {
          requestedPath: null,
        });
        applySettingsForm(root, settings);
        await applySavedLocale(root, settings, invokeFn);
        await refreshExcludedIcons(root, invokeFn);
        await refreshShortcuts?.();
        await refreshExportPreview();
        showExportStatus(root, "settings.import.done");
        await emitMotionChanged({
          reduceMotion: settings?.general?.reduceMotion,
        });
        if (settings?.general?.locale !== before) {
          await emitUiLocaleChanged({ locale: settings.general.locale });
        }
        await publishQueueSort(beforeSort, settings?.copy?.queueSort);
      } catch (error) {
        const code = settingsExportFailureCode(error);
        showExportStatus(root, code ? EXPORT_FAILURE_KEYS[code] : "");
      }
    });
  });

  root.addEventListener?.(LOCALE_APPLIED_EVENT, () => {
    syncTitleEnginePanels(root, settings);
    refreshExportPreview().catch(() => {});
    refreshTitleModelStatus(root, invokeFn, settings).catch(() => {});
    refreshLoginItemStatus(root, invokeFn).catch(() => {});
  });
  await refreshExportPreview();

  root.querySelector("#backup-schedule")?.addEventListener("change", persist);
  root.querySelector("#queue-sort")?.addEventListener("change", persist);
  root.querySelector("#ui-locale")?.addEventListener("change", persist);
  root.querySelector("#title-model")?.addEventListener("change", () => {
    const integration = root.querySelector("#title-integration");
    if (integration) {
      integration.value = "none";
    }
    persist();
  });
  root.querySelector("#title-integration")?.addEventListener("change", persist);
  root
    .querySelector("#title-ollama-model")
    ?.addEventListener("change", persist);
  root.querySelector("#title-hosted-base")?.addEventListener("change", persist);
  root
    .querySelector("#title-hosted-confirmed")
    ?.addEventListener("change", persist);
  root.querySelector("#reduce-motion")?.addEventListener("change", persist);
  root.querySelector("#launch-at-login")?.addEventListener("change", persist);
  root
    .querySelector("[data-import-title-gguf]")
    ?.addEventListener("click", (event) => {
      const button = event.currentTarget;
      runBusy(button, async () => {
        try {
          const dto = await invokeFn("import_title_gguf");
          const name = String(dto?.displayName ?? "");
          const id = String(dto?.id ?? "");
          if (!name || name.includes("/") || id.includes("/")) {
            throw new Error("invalid_custom_gguf");
          }
          settings = {
            ...settings,
            general: {
              ...settings.general,
              titleModel: "custom",
              titleCustomId: id,
              titleCustomName: name,
              titleCustomBytes: Number(dto?.bytes ?? 0),
            },
          };
          applySettingsForm(root, settings);
          await persist();
        } catch (error) {
          if (String(error).includes("cancelled")) {
            return;
          }
          const status = root.querySelector("[data-title-model-status]");
          if (status) {
            status.textContent =
              catalogMessage("settings.field.titleModel.importFailed") ||
              TITLE_MODEL_STATUS_FALLBACK.importFailed;
          }
        }
      });
    });
  root
    .querySelector("[data-clear-hosted-key]")
    ?.addEventListener("click", (event) => {
      const button = event.currentTarget;
      runBusy(button, async () => {
        const provider = selectedTitleModelId(root, settings);
        if (!isHostedTitleModel(provider)) {
          return;
        }
        await invokeFn("clear_hosted_title_key", { provider });
        const keyInput = root.querySelector("#title-hosted-key");
        if (keyInput) {
          keyInput.value = "";
        }
      });
    });
  await refreshEngineExtras();

  root.querySelectorAll("[data-reset-field]").forEach((button) => {
    button.addEventListener("click", () => {
      runBusy(button, async () => {
        const fieldId = button.getAttribute("data-reset-field");
        const before = settings?.general?.locale;
        const beforeSort = settings?.copy?.queueSort;
        settings = await invokeFn("reset_settings_field", { fieldId });
        await applySavedLocale(root, settings, invokeFn);
        await refreshExcludedIcons(root, invokeFn);
        await refreshShortcuts?.();
        await refreshExportPreview();
        await emitMotionChanged({
          reduceMotion: settings?.general?.reduceMotion,
        });
        if (settings?.general?.locale !== before) {
          await emitUiLocaleChanged({ locale: settings.general.locale });
        }
        await publishQueueSort(beforeSort, settings?.copy?.queueSort);
      });
    });
  });
  const resetGroup = root.querySelector("[data-reset-group]");
  resetGroup?.addEventListener("click", () => {
    runBusy(resetGroup, async () => {
      const group = resetGroup.getAttribute("data-reset-group");
      const before = settings?.general?.locale;
      const beforeSort = settings?.copy?.queueSort;
      settings = await invokeFn("reset_settings_group", { group });
      await applySavedLocale(root, settings, invokeFn);
      await refreshExcludedIcons(root, invokeFn);
      await refreshShortcuts?.();
      await refreshExportPreview();
      await emitMotionChanged({
        reduceMotion: settings?.general?.reduceMotion,
      });
      if (settings?.general?.locale !== before) {
        await emitUiLocaleChanged({ locale: settings.general.locale });
      }
      await publishQueueSort(beforeSort, settings?.copy?.queueSort);
    });
  });
  const resetAll = root.querySelector("[data-reset-all]");
  resetAll?.addEventListener("click", () => {
    runBusy(resetAll, async () => {
      const before = settings?.general?.locale;
      const beforeSort = settings?.copy?.queueSort;
      settings = await invokeFn("reset_settings_all");
      await applySavedLocale(root, settings, invokeFn);
      await refreshExcludedIcons(root, invokeFn);
      await refreshShortcuts?.();
      await refreshExportPreview();
      await emitMotionChanged({
        reduceMotion: settings?.general?.reduceMotion,
      });
      if (settings?.general?.locale !== before) {
        await emitUiLocaleChanged({ locale: settings.general.locale });
      }
      await publishQueueSort(beforeSort, settings?.copy?.queueSort);
    });
  });

  root.querySelectorAll("[data-open-window]").forEach((button) => {
    button.addEventListener("click", () => {
      runBusy(button, async () => {
        const kind = button.getAttribute("data-open-window");
        if (kind) {
          await showChromeWindow(kind, invokeFn);
        }
      });
    });
  });
}

function showPickerStatus(root, code) {
  const status = root.querySelector("[data-excluded-apps-picker]");
  if (!status) {
    return;
  }
  if (code === "picked" || code === "picker_cancelled") {
    status.textContent = "";
    status.hidden = true;
    return;
  }
  const key =
    code === "picker_unavailable"
      ? "settings.field.excludedBundleIds.pickerUnavailable"
      : "settings.field.excludedBundleIds.invalidApp";
  status.textContent =
    catalogMessage(key) ||
    (code === "picker_unavailable"
      ? "The app picker is unavailable."
      : "That item is not a valid app.");
  status.hidden = false;
}

function bindExcludedPicker(root, invokeFn, persist) {
  const host = root.querySelector("#excluded-apps");
  const search = root.querySelector("#excluded-apps-search");
  const list = root.querySelector("#excluded-apps-list");
  const empty = root.querySelector("[data-excluded-apps-empty]");
  const choose = root.querySelector("[data-pick-installed-app]");
  const searchWrap = search?.closest?.(".app-picker-search");
  if (!host || !search || !list) {
    return;
  }

  const syncSearchChrome = () => {
    searchWrap?.classList.toggle("is-filled", search.value.trim().length > 0);
  };

  const closeList = () => {
    list.hidden = true;
    search.setAttribute("aria-expanded", "false");
    if (empty) {
      empty.hidden = true;
    }
  };

  const paintOptions = () => {
    if (host._appsUnavailable) {
      closeList();
      return;
    }
    const matches = filterInstalledApps(
      host._installedApps ?? [],
      search.value,
      readExcludedBundleIds(host),
    );
    list.replaceChildren();
    for (const app of matches.slice(0, 50)) {
      const option = globalThis.document.createElement("button");
      option.type = "button";
      option.setAttribute("role", "option");
      option.className = "app-picker-option";
      option.dataset.bundleId = app.bundleId;
      option.dataset.appName = app.name;
      const icon = globalThis.document.createElement("img");
      icon.alt = "";
      icon.className = "app-picker-option-icon";
      icon.width = 16;
      icon.height = 16;
      const label = globalThis.document.createElement("span");
      label.textContent = app.name;
      option.append(icon, label);
      option.addEventListener("click", () => {
        writeExcludedApps(host, [
          ...resolveExcludedApps(
            readExcludedBundleIds(host),
            host._installedApps,
          ),
          app,
        ]);
        search.value = "";
        syncSearchChrome();
        closeList();
        persist();
      });
      list.append(option);
      fillAppIcon(icon, app.bundleId, invokeFn);
    }
    const searching = search.value.trim().length > 0;
    list.hidden = !searching || matches.length === 0;
    search.setAttribute(
      "aria-expanded",
      searching && matches.length > 0 ? "true" : "false",
    );
    if (empty) {
      empty.hidden = !searching || matches.length > 0;
    }
    for (const chip of host.querySelectorAll("[data-excluded-app] img")) {
      fillAppIcon(chip, chip.parentElement?.dataset?.bundleId, invokeFn);
    }
  };

  search.addEventListener("input", () => {
    syncSearchChrome();
    paintOptions();
  });
  search.addEventListener("focus", paintOptions);
  choose?.addEventListener("click", () => {
    runBusy(choose, async () => {
      const result = await pickExcludedApp(host, invokeFn);
      showPickerStatus(root, result);
      if (result === "picked") {
        search.value = "";
        syncSearchChrome();
        closeList();
        persist();
      }
    });
  });
  search.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      closeList();
    }
    if (event.key === "Enter") {
      event.preventDefault();
      list.querySelector("[data-bundle-id]")?.click();
    }
  });
  host.addEventListener("click", (event) => {
    const button = event.target?.closest?.("[data-remove-excluded]");
    if (!button || !host.contains(button)) {
      return;
    }
    const removeId = button.getAttribute("data-remove-excluded");
    writeExcludedApps(
      host,
      resolveExcludedApps(
        readExcludedBundleIds(host),
        host._installedApps,
      ).filter(
        (app) =>
          app.bundleId.toLocaleLowerCase() !== removeId?.toLocaleLowerCase(),
      ),
    );
    persist();
  });
  root.addEventListener?.("pointerdown", (event) => {
    if (!host.contains(event.target)) {
      closeList();
    }
  });
  syncSearchChrome();
  paintOptions();
}

async function refreshExcludedIcons(root, invokeFn) {
  const host = root.querySelector("#excluded-apps");
  if (!host) {
    return;
  }
  for (const chip of host.querySelectorAll("[data-excluded-app] img")) {
    await fillAppIcon(chip, chip.parentElement?.dataset?.bundleId, invokeFn);
  }
}

async function fillAppIcon(img, bundleId, invokeFn) {
  if (!img || !isSafeBundleId(bundleId)) {
    return;
  }
  try {
    const src = sourceIconSrc(
      await invokeFn("app_icon_data_url", { bundleId }),
    );
    if (src) {
      img.src = src;
      img.classList.add("is-ready");
    } else {
      img.classList.remove("is-ready");
    }
  } catch {
    img.classList.remove("is-ready");
  }
}

if (globalThis.document?.readyState) {
  bindSettingsLive();
}
