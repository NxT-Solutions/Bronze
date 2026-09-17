export function readStatusText(root, key, slot = "action-message") {
  const source = root?.querySelector(`[data-${slot}][data-i18n="${key}"]`);
  return source?.textContent?.trim() ?? "";
}

export function applyActionStatus(root, key) {
  const status = root?.querySelector("#action-status");
  if (!status) {
    return;
  }
  const text = readStatusText(root, key);
  status.textContent = text;
  status.hidden = text.length === 0;
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
