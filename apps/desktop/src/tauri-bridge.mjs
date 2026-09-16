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

export function showChromeWindow(kind, invokeFn = tauriInvoke) {
  return invokeFn("show_chrome_window", { kind });
}
