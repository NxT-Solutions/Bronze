export const RETEST_COMMAND = "retest_used_permissions";
export const OPEN_SETTINGS_COMMAND = "open_privacy_settings";

export function tauriInvoke(cmd, args) {
  const invoke = globalThis.__TAURI_INTERNALS__?.invoke;
  if (typeof invoke !== "function") {
    return Promise.reject(new Error("invoke_unavailable"));
  }
  return invoke(cmd, args);
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
  for (const button of root.querySelectorAll("[data-permission-retest]")) {
    button.addEventListener("click", async () => {
      const capability = button.getAttribute("data-permission-retest");
      try {
        const { revealSettings } = await retestUsedPermission(
          capability,
          invokeFn,
        );
        if (revealSettings) {
          const open = root.querySelector(
            `[data-permission-open-settings="${capability}"]`,
          );
          if (open) {
            open.hidden = false;
          }
        }
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
