import { renderMarkdownBody } from "./markdown-body.mjs";

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
