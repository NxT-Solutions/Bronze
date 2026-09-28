import { motionAllowed } from "./control.mjs";
import { renderMarkdownBody } from "./markdown-body.mjs";

export const TITLE_WRITING_DELAY_MS = 80;
export const TITLE_FADE_MS = 200;

export function titleSlotPresentation({
  phase,
  title,
  elapsedMs = 0,
  motion = true,
  reveal = "instant",
} = {}) {
  const pending = phase === "pending";
  const finalTitle = typeof title === "string" ? title.trim() : "";
  if (pending) {
    const showWriting =
      reveal === "instant" || !motion || elapsedMs >= TITLE_WRITING_DELAY_MS;
    return {
      pending: true,
      showTitle: false,
      showWriting,
      showSpinner: Boolean(showWriting && motion),
      title: "",
      fade: false,
    };
  }
  return {
    pending: false,
    showTitle: finalTitle.length > 0,
    showWriting: false,
    showSpinner: false,
    title: finalTitle,
    fade:
      motion &&
      (reveal === "enter" || reveal === "update") &&
      finalTitle.length > 0,
  };
}

const PNG_DATA_PREFIX = "data:image/png;base64,";

export function formatCaptureSource(template, appName) {
  if (typeof appName !== "string") {
    return null;
  }
  const name = appName.trim();
  if (
    name.length === 0 ||
    typeof template !== "string" ||
    !template.includes("{appName}")
  ) {
    return null;
  }
  return template.replaceAll("{appName}", name);
}

export function sourceIconSrc(value) {
  if (typeof value !== "string") {
    return null;
  }
  if (!value.startsWith(PNG_DATA_PREFIX)) {
    return null;
  }
  if (/https?:|file:|javascript:/i.test(value)) {
    return null;
  }
  if (value.length <= PNG_DATA_PREFIX.length) {
    return null;
  }
  return value;
}

export function applySourceRow(article, formattedLabel, iconSrc, options = {}) {
  const source = article?.querySelector("[data-slot=source]");
  if (!source) {
    return;
  }
  const labelEl = source.querySelector("[data-slot=source-label]") ?? source;
  if (!formattedLabel) {
    if (options.preserve) {
      return;
    }
    source.hidden = true;
    return;
  }
  labelEl.textContent = formattedLabel;
  source.hidden = false;
  const img = source.querySelector("[data-slot=source-icon]");
  if (!img) {
    return;
  }
  if (iconSrc === undefined) {
    return;
  }
  const safe = sourceIconSrc(iconSrc);
  if (safe) {
    img.src = safe;
    img.hidden = false;
  } else {
    img.removeAttribute?.("src");
    img.src = "";
    img.hidden = true;
  }
}

export function itemOmitsSource(item) {
  return (
    !item ||
    (!Object.hasOwn(item, "sourceAppName") &&
      !Object.hasOwn(item, "sourceBundleId") &&
      !Object.hasOwn(item, "sourceAppIcon"))
  );
}

export function sourceAppDisplayName(item) {
  if (typeof item?.sourceAppName === "string" && item.sourceAppName.trim()) {
    return item.sourceAppName.trim();
  }
  if (typeof item?.sourceBundleId === "string" && item.sourceBundleId.trim()) {
    return item.sourceBundleId.trim();
  }
  return "";
}

export function sourceLabelTemplate(source, catalogTemplate) {
  const labelEl = source?.querySelector("[data-slot=source-label]") ?? source;
  const candidates = [
    labelEl?.dataset?.sourceTemplate,
    catalogTemplate,
    labelEl?.textContent,
  ];
  for (const value of candidates) {
    if (typeof value === "string" && value.includes("{appName}")) {
      if (labelEl?.dataset) {
        labelEl.dataset.sourceTemplate = value;
      }
      return value;
    }
  }
  return typeof catalogTemplate === "string" ? catalogTemplate : "";
}

export function applyItemSource(article, item, catalogTemplate) {
  if (itemOmitsSource(item)) {
    applySourceRow(article, null, undefined, { preserve: true });
    return;
  }
  const name = sourceAppDisplayName(item);
  if (!name) {
    applySourceRow(article, null, item.sourceAppIcon);
    return;
  }
  const source = article?.querySelector("[data-slot=source]");
  const template = sourceLabelTemplate(source, catalogTemplate);
  const label = formatCaptureSource(template, name);
  applySourceRow(article, label, item.sourceAppIcon, { preserve: !label });
}

export function readExpandLabels(root) {
  const more = root?.querySelector("[data-slot=expand]");
  const less = root?.querySelector("[data-slot=show-less]");
  const sourceLabel = root?.querySelector("[data-slot=source-label]");
  const writing = root?.querySelector("[data-slot=title-writing-label]");
  return {
    showMore: more?.textContent?.trim() ?? "",
    showLess: less?.textContent?.trim() ?? "",
    sourceTemplate: sourceLabel?.textContent?.trim() ?? "",
    writingTitle: writing?.textContent?.trim() ?? "",
  };
}

export const BODY_CLAMP_LINES = 3;
export const BODY_LINE_HEIGHT_EM = 1.45;
export const BODY_EXPAND_MS = 320;
export const BODY_CLAMP_MAX_HEIGHT = `calc(${BODY_CLAMP_LINES} * ${BODY_LINE_HEIGHT_EM}em)`;

function easeOutCubic(t) {
  return 1 - (1 - t) ** 3;
}

export function collapsedBodyMaxHeightPx(body) {
  const view = body?.ownerDocument?.defaultView;
  const style = view?.getComputedStyle?.(body);
  if (style) {
    const fontSize = Number.parseFloat(style.fontSize);
    if (Number.isFinite(fontSize) && fontSize > 0) {
      return BODY_CLAMP_LINES * BODY_LINE_HEIGHT_EM * fontSize;
    }
    const lineHeight = Number.parseFloat(style.lineHeight);
    if (Number.isFinite(lineHeight) && lineHeight > 0) {
      return BODY_CLAMP_LINES * lineHeight;
    }
  }
  if (typeof body?.clientHeight === "number" && body.clientHeight > 0) {
    return body.clientHeight;
  }
  return 0;
}

function parseInlineMaxHeightPx(body) {
  const raw = body?.style?.maxHeight;
  if (typeof raw !== "string" || raw.length === 0 || raw === "none") {
    return null;
  }
  const value = Number.parseFloat(raw);
  return Number.isFinite(value) ? value : null;
}

function clearInlineBodyMaxHeight(body) {
  if (!body?.style) {
    return;
  }
  if (typeof body.style.removeProperty === "function") {
    body.style.removeProperty("max-height");
    body.style.removeProperty("overflow");
    return;
  }
  body.style.maxHeight = "";
  body.style.overflow = "";
}

function cancelBodyExpandFrame(article) {
  const view = article?.ownerDocument?.defaultView;
  const id = article?.dataset?.bodyExpandFrame;
  if (view && id && typeof view.cancelAnimationFrame === "function") {
    view.cancelAnimationFrame(Number(id));
  }
  if (article?.dataset) {
    delete article.dataset.bodyExpandFrame;
  }
}

function applyExpandChrome(article, expanded, labels) {
  const button = article.querySelector("[data-slot=expand]");
  if (!button) {
    return;
  }
  button.setAttribute("aria-expanded", expanded ? "true" : "false");
  button.dataset.i18n = expanded
    ? "queue.item.showLess"
    : "queue.item.showMore";
  const text = expanded ? labels?.showLess : labels?.showMore;
  if (text) {
    button.textContent = text;
  }
}

function finishBodyExpand(article, body, expanded) {
  cancelBodyExpandFrame(article);
  article.classList.remove("is-body-opening", "is-body-closing");
  article.classList.toggle("is-expanded", expanded);
  clearInlineBodyMaxHeight(body);
}

function startBodyExpand(article, body, expanded, labels) {
  const view = article.ownerDocument?.defaultView;
  const clampPx = collapsedBodyMaxHeightPx(body);
  const inlinePx = parseInlineMaxHeightPx(body);
  const from = expanded
    ? (inlinePx ??
      (typeof body.clientHeight === "number" && body.clientHeight > 0
        ? body.clientHeight
        : clampPx))
    : (inlinePx ??
      (typeof body.scrollHeight === "number" && body.scrollHeight > 0
        ? body.scrollHeight
        : clampPx));
  const to = expanded
    ? typeof body.scrollHeight === "number"
      ? body.scrollHeight
      : from
    : clampPx;
  const canAnimate =
    Boolean(view) &&
    typeof view.requestAnimationFrame === "function" &&
    Number.isFinite(from) &&
    Number.isFinite(to) &&
    from !== to;

  if (!canAnimate) {
    finishBodyExpand(article, body, expanded);
    applyExpandChrome(article, expanded, labels);
    return;
  }

  if (!body.style) {
    body.style = {};
  }
  body.style.maxHeight = `${from}px`;
  body.style.overflow = "hidden";
  article.classList.remove("is-body-opening", "is-body-closing");
  article.classList.add(expanded ? "is-body-opening" : "is-body-closing");
  article.classList.toggle("is-expanded", expanded);
  applyExpandChrome(article, expanded, labels);

  let origin = null;
  const tick = (now) => {
    const clock = typeof now === "number" ? now : 0;
    if (origin == null) {
      origin = clock;
    }
    const t = Math.min(1, Math.max(0, (clock - origin) / BODY_EXPAND_MS));
    body.style.maxHeight = `${from + (to - from) * easeOutCubic(t)}px`;
    if (t < 1) {
      article.dataset.bodyExpandFrame = String(
        view.requestAnimationFrame(tick),
      );
      return;
    }
    finishBodyExpand(article, body, expanded);
  };
  article.dataset.bodyExpandFrame = String(view.requestAnimationFrame(tick));
}

export function applyExpandState(article, expanded, labels, options = {}) {
  if (!article) {
    return;
  }
  const body = article.querySelector("[data-slot=body]");
  const motion = options.motion ?? motionAllowed(article.ownerDocument);
  const animate = options.animate === true && motion;
  cancelBodyExpandFrame(article);
  if (animate && body) {
    startBodyExpand(article, body, expanded, labels);
    return;
  }
  article.classList.remove("is-body-opening", "is-body-closing");
  clearInlineBodyMaxHeight(body);
  article.classList.toggle("is-expanded", expanded);
  applyExpandChrome(article, expanded, labels);
}

export function syncExpandVisibility(article) {
  const body = article?.querySelector("[data-slot=body]");
  const button = article?.querySelector("[data-slot=expand]");
  if (!body || !button) {
    return;
  }
  if (
    article.classList.contains("is-expanded") ||
    article.classList.contains("is-body-closing")
  ) {
    button.hidden = false;
    return;
  }
  const overflow =
    typeof body.scrollHeight === "number" &&
    typeof body.clientHeight === "number" &&
    body.scrollHeight > body.clientHeight;
  button.hidden = !overflow;
}

export function wireExpandReader(article, labels) {
  const button = article?.querySelector("[data-slot=expand]");
  if (!button) {
    return;
  }
  applyExpandState(article, article.classList.contains("is-expanded"), labels);
  if (button.dataset.expandWired === "1") {
    return;
  }
  button.dataset.expandWired = "1";
  button.addEventListener("click", () => {
    const next = !article.classList.contains("is-expanded");
    applyExpandState(article, next, labels, { animate: true });
    syncExpandVisibility(article);
  });
}

function clearTitleWritingTimer(article) {
  const view = article.ownerDocument?.defaultView;
  const id = article.dataset?.titleWritingTimer;
  if (view && id && typeof view.clearTimeout === "function") {
    view.clearTimeout(Number(id));
  }
  if (article.dataset) {
    delete article.dataset.titleWritingTimer;
  }
}

function applyTitleSlot(article, item, labels, options = {}) {
  const titleEl = article.querySelector("[data-slot=title]");
  if (!titleEl) {
    return;
  }
  const textEl = titleEl.querySelector?.("[data-slot=title-text]");
  const writingEl = titleEl.querySelector?.("[data-slot=title-writing]");
  const labelEl = titleEl.querySelector?.("[data-slot=title-writing-label]");
  const spinnerEl =
    writingEl?.querySelector("[data-slot=title-spinner]") ??
    writingEl?.querySelector(".queue-title-spinner");
  const writingLabel =
    (typeof labels?.writingTitle === "string" && labels.writingTitle.trim()) ||
    labelEl?.textContent?.trim() ||
    "";

  if (!textEl || !writingEl) {
    const title = typeof item.title === "string" ? item.title.trim() : "";
    if (title.length > 0) {
      titleEl.textContent = title;
      titleEl.hidden = false;
    } else {
      titleEl.textContent = "";
      titleEl.hidden = true;
    }
    return;
  }

  const motion = options.motion ?? motionAllowed(article.ownerDocument);
  const reveal = options.reveal ?? "instant";
  const now =
    typeof options.now === "number"
      ? options.now
      : typeof Date.now === "function"
        ? Date.now()
        : 0;
  const pending = item?.titlePhase === "pending";
  if (pending && !article.dataset.titleWritingAt) {
    article.dataset.titleWritingAt = String(now);
  }
  if (!pending) {
    clearTitleWritingTimer(article);
    delete article.dataset.titleWritingAt;
  }
  const started = Number(article.dataset.titleWritingAt || now);
  const slot = titleSlotPresentation({
    phase: pending ? "pending" : "final",
    title: item?.title,
    elapsedMs: Math.max(0, now - started),
    motion,
    reveal,
  });

  if (pending && !slot.showWriting && motion && reveal === "enter") {
    const view = article.ownerDocument?.defaultView;
    if (
      view &&
      typeof view.setTimeout === "function" &&
      !article.dataset.titleWritingTimer
    ) {
      article.dataset.titleWritingTimer = String(
        view.setTimeout(() => {
          delete article.dataset.titleWritingTimer;
          applyTitleSlot(article, item, labels, {
            ...options,
            now: now + TITLE_WRITING_DELAY_MS,
            reveal: "enter",
          });
        }, TITLE_WRITING_DELAY_MS),
      );
    }
  }

  const previousPhase = article.dataset.titlePhase || "";
  article.dataset.titlePhase = pending ? "pending" : "final";
  article.classList.toggle("is-title-pending", slot.pending);
  const shouldFade =
    slot.fade &&
    ((reveal === "enter" && !slot.pending) || previousPhase === "pending");
  article.classList.toggle("is-title-revealing", shouldFade);

  textEl.textContent = slot.title;
  if (labelEl && writingLabel) {
    labelEl.textContent = writingLabel;
  }
  writingEl.setAttribute("aria-hidden", slot.showWriting ? "false" : "true");
  if (spinnerEl) {
    spinnerEl.hidden = !slot.showSpinner;
  }
  if (slot.pending) {
    titleEl.hidden = false;
    titleEl.setAttribute("aria-busy", "true");
    if (slot.showWriting) {
      titleEl.removeAttribute("aria-label");
    } else if (writingLabel) {
      titleEl.setAttribute("aria-label", writingLabel);
    }
  } else {
    titleEl.hidden = !slot.showTitle;
    titleEl.removeAttribute("aria-busy");
    titleEl.removeAttribute("aria-label");
  }
}

export function fillItemChrome(article, item, labels, options = {}) {
  if (!article) {
    return;
  }
  article.lang = item.contentLanguage || "und";
  article.dir = "auto";
  applyTitleSlot(article, item, labels, options);

  const body = article.querySelector("[data-slot=body]");
  if (body) {
    renderMarkdownBody(body, typeof item.body === "string" ? item.body : "");
  }

  applyItemSource(article, item, labels?.sourceTemplate);
  wireExpandReader(article, labels);
}
