import { catalogMessage } from "./apply-locale.mjs";
import { runBusy } from "./control.mjs";
import { tauriInvoke } from "./tauri-bridge.mjs";

const EXPORT_FAILURE_KEYS = {
  picker_unavailable: "help.diagnostics.export.failed",
  support_secret: "help.diagnostics.export.failed",
  support_path_invalid: "help.diagnostics.export.failed",
  webview_path_rejected: "help.diagnostics.export.failed",
};

export function supportExportFailureCode(error) {
  const message = String(error?.message ?? error ?? "");
  if (message.includes("picker_cancelled")) {
    return "";
  }
  for (const code of Object.keys(EXPORT_FAILURE_KEYS)) {
    if (message.includes(code)) {
      return code;
    }
  }
  return "support_path_invalid";
}

export function showSupportExportStatus(root, key) {
  const status = root.querySelector("[data-export-support-status]");
  if (!status) {
    return;
  }
  if (!key) {
    status.textContent = "";
    status.hidden = true;
    return;
  }
  status.textContent =
    catalogMessage(key) ||
    (key === "help.diagnostics.export.done"
      ? "Saved the support file on this Mac."
      : "The support file could not be saved.");
  status.hidden = status.textContent.length === 0;
}

export async function refreshSupportPreview(root, invokeFn = tauriInvoke) {
  const preview = root.querySelector("[data-diagnostics-preview]");
  if (!preview) {
    return;
  }
  preview.hidden = false;
  preview.removeAttribute("aria-hidden");
  try {
    const text = await invokeFn("preview_support_bundle");
    if (typeof text === "string" && text.length > 0) {
      preview.textContent = text;
    }
  } catch {
    if (!preview.textContent?.trim()) {
      preview.textContent = "unavailable";
    }
  }
}

export function bindHelpSupport(root = document, invokeFn = tauriInvoke) {
  if (!root?.getElementById?.("help")) {
    return;
  }
  const button = root.querySelector("[data-export-support]");
  button?.addEventListener("click", () => {
    runBusy(button, async () => {
      try {
        await invokeFn("export_support_file", { requestedPath: null });
        showSupportExportStatus(root, "help.diagnostics.export.done");
        await refreshSupportPreview(root, invokeFn);
      } catch (error) {
        const code = supportExportFailureCode(error);
        showSupportExportStatus(root, code ? EXPORT_FAILURE_KEYS[code] : "");
      }
    });
  });
  return refreshSupportPreview(root, invokeFn);
}

if (globalThis.document?.readyState) {
  bindHelpSupport();
}
