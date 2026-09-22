import { tauriEmit, tauriInvoke, tauriListen } from "./tauri-bridge.mjs";

export const UI_MOTION_EVENT = "ui-motion-changed";

export function parseReduceMotion(raw) {
  const value = String(raw ?? "").trim();
  if (value === "on" || value === "reduce") {
    return "on";
  }
  if (value === "off") {
    return "off";
  }
  return "system";
}

export function reduceMotionFromSettings(settings) {
  return parseReduceMotion(
    settings?.general?.reduceMotion ?? settings?.accessibility?.motion,
  );
}

export function resolveMotionDataset(pref, systemReduce) {
  const parsed = parseReduceMotion(pref);
  if (parsed === "off") {
    return "full";
  }
  if (parsed === "on") {
    return "reduce";
  }
  return systemReduce ? "reduce" : "full";
}

export function readSystemReduce(doc) {
  return Boolean(
    doc?.defaultView?.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches,
  );
}

export function applyMotionDataset(root, pref, systemReduce) {
  const html = root?.documentElement ?? root;
  if (!html?.dataset) {
    return "";
  }
  const motion = resolveMotionDataset(pref, systemReduce);
  html.dataset.motion = motion;
  if (typeof html.toggleAttribute === "function") {
    html.toggleAttribute("data-reduce-motion", motion === "reduce");
  } else if (motion === "reduce") {
    html.setAttribute?.("data-reduce-motion", "");
  } else {
    html.removeAttribute?.("data-reduce-motion");
  }
  return motion;
}

export function applyMotionFromSettings(root, settings) {
  const doc = root?.documentElement ? root : (root?.ownerDocument ?? document);
  return applyMotionDataset(
    doc,
    reduceMotionFromSettings(settings),
    readSystemReduce(doc),
  );
}

export function emitMotionChanged(payload = {}) {
  try {
    const channel = new BroadcastChannel(UI_MOTION_EVENT);
    channel.postMessage(payload);
    channel.close();
  } catch {
    // BroadcastChannel is absent in some test runtimes.
  }
  return tauriEmit(UI_MOTION_EVENT, payload);
}

export function listenMotionChanged(handler) {
  let channel;
  try {
    channel = new BroadcastChannel(UI_MOTION_EVENT);
    channel.onmessage = (event) => {
      handler(event.data ?? {});
    };
  } catch {
    channel = null;
  }
  tauriListen(UI_MOTION_EVENT, (event) => {
    handler(event?.payload ?? event ?? {});
  });
  return () => {
    channel?.close();
  };
}

let motionPref = "system";

export async function bindMotionPreference(
  root = document,
  invokeFn = tauriInvoke,
) {
  const doc = root?.documentElement ? root : (root?.ownerDocument ?? document);
  let media;
  const apply = () =>
    applyMotionDataset(doc, motionPref, Boolean(media?.matches));

  try {
    media = doc.defaultView?.matchMedia?.("(prefers-reduced-motion: reduce)");
    media?.addEventListener?.("change", apply);
  } catch {
    media = null;
  }

  async function load() {
    try {
      const settings = await invokeFn("load_settings_v1");
      motionPref = reduceMotionFromSettings(settings);
    } catch {
      motionPref = "system";
    }
    apply();
    return motionPref;
  }

  listenMotionChanged((payload) => {
    if (payload?.reduceMotion != null) {
      motionPref = parseReduceMotion(payload.reduceMotion);
      apply();
      return;
    }
    load();
  });

  await load();
  return { reload: load, apply };
}
