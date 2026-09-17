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

export function tauriListen(event, handler) {
  const listen = globalThis.__TAURI__?.event?.listen;
  if (typeof listen === "function") {
    return listen(event, handler);
  }
  return Promise.resolve(null);
}

export function tauriEmit(event, payload) {
  const emit = globalThis.__TAURI__?.event?.emit;
  if (typeof emit === "function") {
    return emit(event, payload);
  }
  return Promise.resolve(null);
}
