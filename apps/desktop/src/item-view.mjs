import { renderMarkdownBody } from "./markdown-body.mjs";

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

export function applySourceRow(article, formattedLabel, iconSrc) {
  const source = article?.querySelector("[data-slot=source]");
  if (!source) {
    return;
  }
  const labelEl = source.querySelector("[data-slot=source-label]") ?? source;
  if (!formattedLabel) {
    source.hidden = true;
    return;
  }
  labelEl.textContent = formattedLabel;
  source.hidden = false;
  const img = source.querySelector("[data-slot=source-icon]");
  if (!img) {
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

export function readExpandLabels(root) {
  const more = root?.querySelector("[data-slot=expand]");
  const less = root?.querySelector("[data-slot=show-less]");
  return {
    showMore: more?.textContent?.trim() ?? "",
    showLess: less?.textContent?.trim() ?? "",
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
  applyExpandState(article, false, labels);
  button.addEventListener("click", () => {
    const next = !article.classList.contains("is-expanded");
    applyExpandState(article, next, labels);
    syncExpandVisibility(article);
  });
}

export function fillItemChrome(article, item, labels) {
  if (!article) {
    return;
  }
  article.lang = item.contentLanguage || "und";
  article.dir = "auto";

  const titleEl = article.querySelector("[data-slot=title]");
  const title = typeof item.title === "string" ? item.title.trim() : "";
  if (titleEl) {
    if (title.length > 0) {
      titleEl.textContent = title;
      titleEl.hidden = false;
    } else {
      titleEl.textContent = "";
      titleEl.hidden = true;
    }
  }

  const body = article.querySelector("[data-slot=body]");
  if (body) {
    renderMarkdownBody(body, typeof item.body === "string" ? item.body : "");
  }

  wireExpandReader(article, labels);
}
