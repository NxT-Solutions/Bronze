export const RETEST_COMMAND = "retest_used_permissions";
export const OPEN_SETTINGS_COMMAND = "open_privacy_settings";

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
      if (!isUsedCapability(capability)) {
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
