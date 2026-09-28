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

export function applyExpandState(article, expanded, labels) {
  if (!article) {
    return;
  }
  article.classList.toggle("is-expanded", expanded);
  const button = article.querySelector("[data-slot=expand]");
  if (!button) {
    return;
  }
  button.setAttribute("aria-expanded", expanded ? "true" : "false");
  button.dataset.i18n = expanded
    ? "queue.item.showLess"
    : "queue.item.showMore";
  const text = expanded ? labels.showLess : labels.showMore;
  if (text) {
    button.textContent = text;
  }
}

export function syncExpandVisibility(article) {
  const body = article?.querySelector("[data-slot=body]");
  const button = article?.querySelector("[data-slot=expand]");
  if (!body || !button) {
    return;
  }
  if (article.classList.contains("is-expanded")) {
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
    applyExpandState(article, next, labels);
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
  const textEl = titleEl.querySelector("[data-slot=title-text]");
  const writingEl = titleEl.querySelector("[data-slot=title-writing]");
  const labelEl = titleEl.querySelector("[data-slot=title-writing-label]");
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
