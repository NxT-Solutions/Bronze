const TIP_MS = 2200;

export const MOTION = {
  duration: 180,
  easing: "cubic-bezier(0.22, 1, 0.36, 1)",
};

export function motionAllowed(doc) {
  if (!doc) {
    return false;
  }
  const motion = doc.documentElement?.dataset?.motion;
  if (motion === "full") {
    return true;
  }
  if (motion === "reduce") {
    return false;
  }
  if (doc.documentElement?.hasAttribute("data-reduce-motion")) {
    return false;
  }
  const query = doc.defaultView?.matchMedia?.(
    "(prefers-reduced-motion: reduce)",
  );
  return !query?.matches;
}

export function animateElement(el, keyframes, options = {}) {
  if (!el || typeof el.animate !== "function") {
    return null;
  }
  if (!motionAllowed(el.ownerDocument)) {
    return null;
  }
  return el.animate(keyframes, {
    duration: MOTION.duration,
    easing: MOTION.easing,
    fill: "forwards",
    ...options,
  });
}

export function readStatusText(root, key, slot = "action-message") {
  const source = root?.querySelector(`[data-${slot}][data-i18n="${key}"]`);
  return source?.textContent?.trim() ?? "";
}

export function applyActionTip(anchor, text, tone = "ok") {
  const host = anchor?.closest?.(".row-actions");
  const tip = host?.querySelector?.("[data-slot=action-tip]");
  if (!tip) {
    return;
  }
  const view = anchor.ownerDocument?.defaultView;
  if (view && tip.dataset.tipTimer) {
    view.clearTimeout(Number(tip.dataset.tipTimer));
    delete tip.dataset.tipTimer;
  }
  tip.textContent = text;
  tip.hidden = text.length === 0;
  if (tone === "failed") {
    tip.dataset.tone = "failed";
  } else {
    delete tip.dataset.tone;
  }
  if (text) {
    animateElement(tip, [
      { opacity: 0, transform: "translateY(4px) scale(0.96)" },
      { opacity: 1, transform: "none" },
    ]);
  }
  if (text && typeof view?.setTimeout === "function") {
    tip.dataset.tipTimer = String(
      view.setTimeout(() => {
        tip.textContent = "";
        tip.hidden = true;
        delete tip.dataset.tone;
        delete tip.dataset.tipTimer;
      }, TIP_MS),
    );
  }
}

export function applyActionStatus(root, key, anchor) {
  const status = root?.querySelector("#action-status");
  const text = readStatusText(root, key);
  if (status) {
    status.textContent = text;
    status.hidden = text.length === 0;
  }
  if (anchor) {
    applyActionTip(anchor, text, key.endsWith("failed") ? "failed" : "ok");
    return;
  }
  showChromeNotice(root, text, key.endsWith("failed") ? "failed" : "ok");
}

export function hideChromeNotice(root) {
  const host = root?.querySelector?.("#chrome-notice");
  if (!host) {
    return;
  }
  const view = host.ownerDocument?.defaultView;
  if (view && host.dataset.noticeTimer) {
    view.clearTimeout(Number(host.dataset.noticeTimer));
    delete host.dataset.noticeTimer;
  }
  const label = host.querySelector?.("#chrome-notice-text");
  if (label) {
    label.textContent = "";
  }
  host.hidden = true;
  delete host.dataset.tone;
  delete host.dataset.noticeItem;
}

export function showChromeNotice(root, text, tone = "ok", itemId = "") {
  const host = root?.querySelector?.("#chrome-notice");
  const label = host?.querySelector?.("#chrome-notice-text");
  if (!host || !label) {
    return;
  }
  const view = host.ownerDocument?.defaultView;
  if (view && host.dataset.noticeTimer) {
    view.clearTimeout(Number(host.dataset.noticeTimer));
    delete host.dataset.noticeTimer;
  }
  label.textContent = text;
  if (itemId) {
    host.dataset.noticeItem = itemId;
  } else {
    delete host.dataset.noticeItem;
  }
  host.hidden = text.length === 0;
  if (tone === "failed") {
    host.dataset.tone = "failed";
  } else {
    delete host.dataset.tone;
  }
  if (text) {
    animateElement(host, [
      { opacity: 0, transform: "translateY(6px)" },
      { opacity: 1, transform: "none" },
    ]);
  }
  if (text && typeof view?.setTimeout === "function") {
    host.dataset.noticeTimer = String(
      view.setTimeout(() => {
        hideChromeNotice(root);
      }, TIP_MS),
    );
  }
}

export function bindChromeNotice(root) {
  if (!root?.addEventListener) {
    return;
  }
  root.addEventListener("click", (event) => {
    if (event.target?.closest?.("[data-notice-dismiss]")) {
      hideChromeNotice(root);
      return;
    }
    const opener = event.target?.closest?.("#chrome-notice-text");
    const host = root.querySelector?.("#chrome-notice");
    const id = host?.dataset?.noticeItem;
    if (opener && id) {
      root.dispatchEvent(
        new CustomEvent("bronze-notice-open", { detail: { id } }),
      );
    }
  });
  root.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      hideChromeNotice(root);
    }
  });
}

export function closeOverflowMenus(root, keep = null) {
  const nodes = root?.querySelectorAll?.("[data-slot=toolbar-overflow]");
  if (!nodes) {
    return;
  }
  for (const el of nodes) {
    if (el !== keep && "open" in el) {
      el.open = false;
    }
  }
}

export function bindIconTips(root) {
  if (!root?.addEventListener) {
    return;
  }
  const clearDismiss = (event) => {
    const host = event.target?.closest?.("[data-slot=action-icons]");
    if (!host?.dataset) {
      return;
    }
    delete host.dataset.tipsDismissed;
  };
  root.addEventListener("pointerdown", clearDismiss);
  root.addEventListener("focusin", clearDismiss);
  root.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") {
      return;
    }
    const host = event.target?.closest?.("[data-slot=action-icons]");
    if (!host?.dataset) {
      return;
    }
    host.dataset.tipsDismissed = "true";
  });
}

export function bindOverflowDismiss(root) {
  if (!root?.addEventListener) {
    return;
  }
  root.addEventListener("pointerdown", (event) => {
    const keep =
      event.target?.closest?.("[data-slot=toolbar-overflow]") ?? null;
    closeOverflowMenus(root, keep);
  });
  root.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") {
      return;
    }
    const open =
      event.target?.closest?.("[data-slot=toolbar-overflow][open]") ??
      root.querySelector?.("[data-slot=toolbar-overflow][open]") ??
      null;
    closeOverflowMenus(root);
    open?.querySelector?.("summary")?.focus?.();
  });
}

export function openEditSheet(root, initial = "") {
  const dialog = root?.querySelector?.("#edit-sheet");
  const field = root?.querySelector?.("#edit-body");
  if (!dialog || !field) {
    return Promise.resolve(null);
  }
  field.value = initial;
  if (typeof dialog.showModal !== "function") {
    return Promise.resolve(null);
  }
  dialog.returnValue = "";
  const dismiss = dialog.querySelector?.("[data-edit-dismiss]");
  const onDismiss = () => {
    dialog.close("cancel");
  };
  dismiss?.addEventListener?.("click", onDismiss);
  return new Promise((resolve) => {
    const finish = () => {
      dialog.removeEventListener("close", finish);
      dismiss?.removeEventListener?.("click", onDismiss);
      resolve(dialog.returnValue === "save" ? field.value : null);
    };
    dialog.addEventListener("close", finish);
    dialog.showModal();
    field.focus?.();
  });
}

export async function runBusy(el, work) {
  if (!el) {
    return work();
  }
  if (el.getAttribute("aria-busy") === "true") {
    return undefined;
  }
  el.setAttribute("aria-busy", "true");
  const canDisable = "disabled" in el;
  if (canDisable) {
    el.disabled = true;
  }
  try {
    return await work();
  } finally {
    el.removeAttribute("aria-busy");
    if (canDisable) {
      el.disabled = false;
    }
  }
}
