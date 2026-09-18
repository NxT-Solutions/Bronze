import { catalogMessage, LOCALE_APPLIED_EVENT } from "./apply-locale.mjs";

export const RETEST_COMMAND = "retest_used_permissions";
export const OPEN_SETTINGS_COMMAND = "open_privacy_settings";
export const NOTICE_STATUS_COMMAND = "notification_authorization_status";
export const NOTICE_REQUEST_COMMAND = "request_notification_authorization";

const NOTICE_STATUS_KEYS = {
  healthy: "settings.permission.status.granted",
  granted: "settings.permission.status.granted",
  denied: "settings.permission.status.denied",
  not_requested: "settings.permission.status.notRequested",
  unavailable: "settings.permission.status.unavailable",
};

const NOTICE_STATUS_FALLBACK = {
  healthy: "Granted",
  granted: "Granted",
  denied: "Denied",
  not_requested: "Not requested",
  unavailable: "Unavailable",
};

export function tauriInvoke(cmd, args) {
  const tauri = globalThis.__TAURI__;
  if (tauri?.core && typeof tauri.core.invoke === "function") {
    return tauri.core.invoke(cmd, args);
  }
  const internals = globalThis.__TAURI_INTERNALS__;
  if (typeof internals?.invoke === "function") {
    return internals.invoke(cmd, args);
  }
  return Promise.reject(new Error("invoke_unavailable"));
}

export function pillStatusForState(state) {
  if (state === "granted_unverified" || state === "healthy") {
    return "granted";
  }
  if (state === "notUsed") {
    return "notUsed";
  }
  return "denied";
}

export function applyPermissionResult(root, result) {
  const rows = [
    ["inputMonitoring", result.input_monitoring],
    ["accessibility", result.accessibility],
  ];
  for (const [capability, state] of rows) {
    const card = root.querySelector(`[data-capability="${capability}"]`);
    if (!card) {
      continue;
    }
    const status = pillStatusForState(state);
    card.dataset.status = status;
    const pill = card.querySelector(".pill");
    if (pill) {
      pill.dataset.status = status;
      if (status === "granted") {
        pill.textContent = "Granted";
      } else if (status === "denied") {
        pill.textContent = "Denied";
      }
    }
    const open = root.querySelector(
      `[data-permission-open-settings="${capability}"]`,
    );
    if (open) {
      open.hidden = !shouldRevealSystemSettings(capability, result);
    }
  }
}

export function isUsedCapability(capability) {
  return capability === "inputMonitoring" || capability === "accessibility";
}

export function isNoticeCapability(capability) {
  return capability === "notifications";
}

export function noticePillStatus(status) {
  if (status === "healthy" || status === "granted") {
    return "granted";
  }
  if (status === "denied") {
    return "denied";
  }
  if (status === "not_requested") {
    return "notRequested";
  }
  return "unavailable";
}

export function shouldRevealNoticeAllow(status) {
  return status === "not_requested";
}

export function shouldRevealNoticeSettings(status) {
  return status === "denied";
}

export function noticeStatusLabel(status) {
  const key = NOTICE_STATUS_KEYS[status] ?? NOTICE_STATUS_KEYS.unavailable;
  return catalogMessage(key) || NOTICE_STATUS_FALLBACK[status] || "Unavailable";
}

export function applyNoticeAuthorization(root, status) {
  const card = root.querySelector('[data-capability="notifications"]');
  if (!card) {
    return;
  }
  const pillStatus = noticePillStatus(status);
  card.dataset.status = pillStatus;
  const pill = card.querySelector(".pill");
  if (pill) {
    pill.dataset.status = pillStatus;
    const key = NOTICE_STATUS_KEYS[status] ?? NOTICE_STATUS_KEYS.unavailable;
    pill.setAttribute("data-i18n", key);
    pill.textContent = noticeStatusLabel(status);
  }
  const allow = root.querySelector("[data-permission-allow-notifications]");
  if (allow) {
    allow.hidden = !shouldRevealNoticeAllow(status);
  }
  const open = root.querySelector(
    '[data-permission-open-settings="notifications"]',
  );
  if (open) {
    open.hidden = !shouldRevealNoticeSettings(status);
  }
}

export async function loadNoticeAuthorization(root, invokeFn = tauriInvoke) {
  const status = await invokeFn(NOTICE_STATUS_COMMAND);
  applyNoticeAuthorization(root, status);
  return status;
}

export async function requestNoticeAuthorization(root, invokeFn = tauriInvoke) {
  const status = await invokeFn(NOTICE_REQUEST_COMMAND);
  applyNoticeAuthorization(root, status);
  return status;
}

export function shouldRevealSystemSettings(capability, result) {
  if (!isUsedCapability(capability)) {
    return false;
  }
  if (result.screen_recording_requested) {
    return false;
  }
  const state =
    capability === "inputMonitoring"
      ? result.input_monitoring
      : result.accessibility;
  return state !== "granted_unverified" && state !== "healthy";
}

export async function retestUsedPermission(capability, invokeFn = tauriInvoke) {
  if (!isUsedCapability(capability)) {
    throw new Error("capability_not_used");
  }
  const result = await invokeFn(RETEST_COMMAND);
  return {
    result,
    revealSettings: shouldRevealSystemSettings(capability, result),
  };
}

export function bindPermissionHealth(root = document, invokeFn = tauriInvoke) {
  invokeFn(RETEST_COMMAND)
    .then((result) => applyPermissionResult(root, result))
    .catch(() => {});
  loadNoticeAuthorization(root, invokeFn).catch(() => {
    applyNoticeAuthorization(root, "unavailable");
  });
  if (globalThis.document) {
    globalThis.document.addEventListener(LOCALE_APPLIED_EVENT, () => {
      const status = root.querySelector('[data-capability="notifications"]')
        ?.dataset?.status;
      if (status === "granted") {
        applyNoticeAuthorization(root, "healthy");
      } else if (status === "notRequested") {
        applyNoticeAuthorization(root, "not_requested");
      } else if (status) {
        applyNoticeAuthorization(root, status);
      }
    });
  }
  const allow = root.querySelector("[data-permission-allow-notifications]");
  allow?.addEventListener("click", async () => {
    try {
      await requestNoticeAuthorization(root, invokeFn);
    } catch {
      applyNoticeAuthorization(root, "unavailable");
    }
  });
  for (const button of root.querySelectorAll("[data-permission-retest]")) {
    button.addEventListener("click", async () => {
      const capability = button.getAttribute("data-permission-retest");
      try {
        const { result } = await retestUsedPermission(capability, invokeFn);
        applyPermissionResult(root, result);
      } catch {
        // Denial still leaves the manual composer path.
      }
    });
  }
  for (const button of root.querySelectorAll(
    "[data-permission-open-settings]",
  )) {
    button.addEventListener("click", async () => {
      const capability = button.getAttribute("data-permission-open-settings");
      if (!isUsedCapability(capability) && !isNoticeCapability(capability)) {
        return;
      }
      try {
        await invokeFn(OPEN_SETTINGS_COMMAND, { capability });
      } catch {
        // Keep the composer available if the OS pane cannot open.
      }
    });
  }
}

if (globalThis.document?.readyState) {
  bindPermissionHealth();
}
