import assert from "node:assert/strict";
import { test } from "node:test";
import {
  applyExpandState,
  applyItemSource,
  applySourceRow,
  BODY_CLAMP_MAX_HEIGHT,
  BODY_EXPAND_MS,
  BODY_LINE_HEIGHT_EM,
  collapsedBodyMaxHeightPx,
  fillItemChrome,
  formatCaptureSource,
  readExpandLabels,
  sourceIconSrc,
  syncExpandVisibility,
  TITLE_WRITING_DELAY_MS,
  titleSlotPresentation,
} from "./item-view.mjs";

function createDocument() {
  function createElement(tagName) {
    const children = [];
    const attrs = {};
    const listeners = [];
    const el = {
      nodeType: 1,
      tagName: tagName.toUpperCase(),
      children,
      childNodes: children,
      hidden: false,
      dataset: {},
      className: "",
      lang: "",
      dir: "",
      scrollHeight: 0,
      clientHeight: 0,
      style: {
        maxHeight: "",
        overflow: "",
        height: "",
      },
      ownerDocument: doc,
      get classList() {
        return {
          contains: (name) => el.className.split(/\s+/).includes(name),
          add(...names) {
            for (const name of names) {
              if (!this.contains(name)) {
                el.className = `${el.className} ${name}`.trim();
              }
            }
          },
          remove(...names) {
            el.className = el.className
              .split(/\s+/)
              .filter((part) => part && !names.includes(part))
              .join(" ");
          },
          toggle(name, force) {
            const on = force ?? !this.contains(name);
            if (on) {
              this.add(name);
            } else {
              this.remove(name);
            }
            return on;
          },
        };
      },
      appendChild(child) {
        children.push(child);
        return child;
      },
      append(...nodes) {
        children.push(...nodes);
      },
      replaceChildren(...nodes) {
        children.length = 0;
        children.push(...nodes);
      },
      src: "",
      setAttribute(name, value) {
        attrs[name] = String(value);
        if (name === "src") {
          el.src = String(value);
        }
      },
      getAttribute(name) {
        if (name === "src") {
          return el.src || null;
        }
        return Object.hasOwn(attrs, name) ? attrs[name] : null;
      },
      removeAttribute(name) {
        delete attrs[name];
        if (name === "src") {
          el.src = "";
        }
      },
      addEventListener(type, handler) {
        listeners.push({ type, handler });
      },
      click() {
        for (const listener of listeners) {
          if (listener.type === "click") {
            listener.handler();
          }
        }
      },
      querySelector(sel) {
        return queryAll(el, sel)[0] ?? null;
      },
      querySelectorAll(sel) {
        return queryAll(el, sel);
      },
    };
    Object.defineProperty(el, "textContent", {
      get() {
        if (children.length === 0) {
          return el._text ?? "";
        }
        return children.map((child) => child.textContent ?? "").join("");
      },
      set(value) {
        children.length = 0;
        el._text = String(value);
        if (value) {
          children.push(doc.createTextNode(String(value)));
        }
      },
    });
    return el;
  }

  function walk(node, visit) {
    for (const child of node.children ?? []) {
      visit(child);
      walk(child, visit);
    }
  }

  function queryAll(rootEl, sel) {
    const found = [];
    const slot = sel.match(/data-slot=(?:"([^"]+)"|([A-Za-z0-9-]+))/);
    walk(rootEl, (node) => {
      if (node.nodeType !== 1) {
        return;
      }
      if (slot) {
        if (node.dataset.slot === (slot[1] ?? slot[2])) {
          found.push(node);
        }
        return;
      }
      if (node.tagName === sel.toUpperCase()) {
        found.push(node);
      }
    });
    return found;
  }

  const timers = [];
  const doc = {
    documentElement: { dataset: {}, hasAttribute: () => false },
    defaultView: {
      setTimeout(fn, ms) {
        const id = timers.length + 1;
        timers.push({ id, fn, ms });
        return id;
      },
      clearTimeout(id) {
        const at = timers.findIndex((timer) => timer.id === id);
        if (at >= 0) {
          timers.splice(at, 1);
        }
      },
    },
    flushTimers() {
      const pending = timers.splice(0, timers.length);
      for (const timer of pending) {
        timer.fn();
      }
    },
    createElement,
    createTextNode(text) {
      return {
        nodeType: 3,
        nodeName: "#text",
        data: String(text),
        get textContent() {
          return this.data;
        },
        children: [],
      };
    },
  };
  return doc;
}

function articleFixture(doc) {
  const article = doc.createElement("article");
  const title = doc.createElement("h3");
  title.dataset.slot = "title";
  title.hidden = true;
  const body = doc.createElement("div");
  body.dataset.slot = "body";
  const expand = doc.createElement("button");
  expand.dataset.slot = "expand";
  expand.dataset.i18n = "queue.item.showMore";
  expand.textContent = "Show more";
  expand.hidden = true;
  expand.setAttribute("aria-expanded", "false");
  const less = doc.createElement("span");
  less.dataset.slot = "show-less";
  less.textContent = "Show less";
  less.hidden = true;
  const source = doc.createElement("p");
  source.dataset.slot = "source";
  source.hidden = true;
  const icon = doc.createElement("img");
  icon.dataset.slot = "source-icon";
  icon.hidden = true;
  const label = doc.createElement("span");
  label.dataset.slot = "source-label";
  label.textContent = "From {appName}";
  source.append(icon, label);
  article.append(title, body, expand, less, source);
  return article;
}

function writingArticleFixture(doc) {
  const article = articleFixture(doc);
  const title = article.querySelector("[data-slot=title]");
  const text = doc.createElement("span");
  text.dataset.slot = "title-text";
  const writing = doc.createElement("span");
  writing.dataset.slot = "title-writing";
  writing.setAttribute("aria-hidden", "true");
  const spinner = doc.createElement("span");
  spinner.dataset.slot = "title-spinner";
  spinner.className = "queue-title-spinner";
  spinner.hidden = true;
  const label = doc.createElement("span");
  label.dataset.slot = "title-writing-label";
  label.textContent = "Writing title…";
  writing.append(spinner, label);
  title.append(text, writing);
  return article;
}

test("fillItemChrome sets a visible title and sanitizes the body", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  const labels = readExpandLabels(article);
  assert.equal(labels.showMore, "Show more");
  assert.equal(labels.showLess, "Show less");
  fillItemChrome(
    article,
    {
      title: "Migration timeout",
      body: "**Hello**\n<script>alert(1)</script>",
      contentLanguage: "en",
    },
    labels,
  );
  const title = article.querySelector("[data-slot=title]");
  const body = article.querySelector("[data-slot=body]");
  assert.equal(title.hidden, false);
  assert.equal(title.textContent, "Migration timeout");
  assert.equal(article.lang, "en");
  assert.equal(body.querySelector("strong").textContent, "Hello");
  assert.equal(body.querySelector("script"), null);
  assert.match(body.textContent, /<script>alert\(1\)<\/script>/);
});

test("fillItemChrome shows a stored title in full without an ellipsis glyph", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  const labels = readExpandLabels(article);
  const title = "Migration timeout is the real bug in";
  fillItemChrome(article, { title, body: "plain" }, labels);
  const heading = article.querySelector("[data-slot=title]");
  assert.equal(heading.hidden, false);
  assert.equal(heading.textContent, title);
  assert.doesNotMatch(heading.textContent, /…|\.\.\./);
});

test("missing or blank titles stay hidden and are not invented", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(article, { title: null, body: "plain" }, labels);
  assert.equal(article.querySelector("[data-slot=title]").hidden, true);
  assert.equal(article.querySelector("[data-slot=title]").textContent, "");
  fillItemChrome(article, { title: "   ", body: "plain" }, labels);
  assert.equal(article.querySelector("[data-slot=title]").hidden, true);
});

test("expand toggle adds is-expanded and swaps catalog labels", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(article, { title: "T", body: "body" }, labels);
  const button = article.querySelector("[data-slot=expand]");
  assert.equal(button.tagName, "BUTTON");
  applyExpandState(article, true, labels);
  assert.equal(article.classList.contains("is-expanded"), true);
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.equal(button.dataset.i18n, "queue.item.showLess");
  assert.equal(button.textContent, "Show less");
  applyExpandState(article, false, labels);
  assert.equal(article.classList.contains("is-expanded"), false);
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(button.dataset.i18n, "queue.item.showMore");
  assert.equal(button.textContent, "Show more");
  button.click();
  assert.equal(article.classList.contains("is-expanded"), true);
  assert.equal(button.textContent, "Show less");
});

test("expand button stays hidden until the body overflows", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  const body = article.querySelector("[data-slot=body]");
  const button = article.querySelector("[data-slot=expand]");
  body.scrollHeight = 20;
  body.clientHeight = 20;
  syncExpandVisibility(article);
  assert.equal(button.hidden, true);
  body.scrollHeight = 80;
  body.clientHeight = 20;
  syncExpandVisibility(article);
  assert.equal(button.hidden, false);
});

function installMotionClock(doc) {
  const frames = [];
  let nextId = 1;
  doc.documentElement.dataset.motion = "full";
  doc.defaultView.getComputedStyle = () => ({
    fontSize: "13px",
    lineHeight: `${13 * BODY_LINE_HEIGHT_EM}px`,
  });
  doc.defaultView.requestAnimationFrame = (fn) => {
    const id = nextId;
    nextId += 1;
    frames.push({ id, fn });
    return id;
  };
  doc.defaultView.cancelAnimationFrame = (id) => {
    const at = frames.findIndex((frame) => frame.id === id);
    if (at >= 0) {
      frames.splice(at, 1);
    }
  };
  doc.flushFrame = (now) => {
    const pending = frames.splice(0, frames.length);
    for (const frame of pending) {
      frame.fn(now);
    }
  };
  return doc;
}

function overflowArticle(doc) {
  const article = articleFixture(doc);
  const body = article.querySelector("[data-slot=body]");
  body.scrollHeight = 240;
  body.clientHeight = 3 * BODY_LINE_HEIGHT_EM * 13;
  return { article, body };
}

function maxHeightPx(body) {
  return Number.parseFloat(body.style.maxHeight);
}

test("expand adds an opening class and does not assign full height in one step", () => {
  const doc = installMotionClock(createDocument());
  const { article, body } = overflowArticle(doc);
  const labels = readExpandLabels(article);
  const clamp = collapsedBodyMaxHeightPx(body);
  fillItemChrome(article, { title: "T", body: "long body" }, labels);
  applyExpandState(article, true, labels, { animate: true, motion: true });
  assert.equal(article.classList.contains("is-body-opening"), true);
  assert.equal(article.classList.contains("is-expanded"), true);
  assert.equal(article.classList.contains("is-sort-expand"), false);
  assert.equal(body.style.height, "");
  assert.equal(body.style.maxHeight, `${clamp}px`);
  assert.notEqual(body.style.maxHeight, `${body.scrollHeight}px`);
  doc.flushFrame(0);
  assert.equal(article.classList.contains("is-body-opening"), true);
  assert.equal(maxHeightPx(body), clamp);
  doc.flushFrame(BODY_EXPAND_MS / 2);
  const mid = maxHeightPx(body);
  assert.ok(mid > clamp);
  assert.ok(mid < body.scrollHeight);
  assert.equal(body.style.height, "");
  doc.flushFrame(BODY_EXPAND_MS);
  assert.equal(article.classList.contains("is-body-opening"), false);
  assert.equal(article.classList.contains("is-expanded"), true);
  assert.equal(body.style.maxHeight, "");
  assert.equal(body.style.height, "");
});

test("collapse adds a closing class and eases back to the three-line clamp", () => {
  const doc = installMotionClock(createDocument());
  const { article, body } = overflowArticle(doc);
  const labels = readExpandLabels(article);
  const clamp = collapsedBodyMaxHeightPx(body);
  fillItemChrome(article, { title: "T", body: "long body" }, labels);
  applyExpandState(article, true, labels, { animate: false, motion: true });
  applyExpandState(article, false, labels, { animate: true, motion: true });
  assert.equal(article.classList.contains("is-body-closing"), true);
  assert.equal(article.classList.contains("is-expanded"), false);
  assert.equal(article.classList.contains("is-sort-expand"), false);
  assert.equal(body.style.height, "");
  assert.equal(body.style.maxHeight, `${body.scrollHeight}px`);
  assert.notEqual(body.style.maxHeight, `${clamp}px`);
  doc.flushFrame(0);
  assert.equal(maxHeightPx(body), body.scrollHeight);
  doc.flushFrame(BODY_EXPAND_MS / 2);
  const mid = maxHeightPx(body);
  assert.ok(mid < body.scrollHeight);
  assert.ok(mid > clamp);
  doc.flushFrame(BODY_EXPAND_MS);
  assert.equal(article.classList.contains("is-body-closing"), false);
  assert.equal(article.classList.contains("is-expanded"), false);
  assert.equal(body.style.maxHeight, "");
  assert.equal(BODY_CLAMP_MAX_HEIGHT, "calc(3 * 1.45em)");
});

test("Show more click uses the opening class when motion is on", () => {
  const doc = installMotionClock(createDocument());
  const { article, body } = overflowArticle(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(article, { title: "T", body: "long body" }, labels);
  const button = article.querySelector("[data-slot=expand]");
  button.click();
  assert.equal(article.classList.contains("is-body-opening"), true);
  assert.equal(article.classList.contains("is-sort-expand"), false);
  assert.equal(body.style.maxHeight, `${collapsedBodyMaxHeightPx(body)}px`);
  assert.equal(body.style.height, "");
});

test("reduced motion snaps Show more open and closed", () => {
  const doc = installMotionClock(createDocument());
  doc.documentElement.dataset.motion = "reduce";
  const { article, body } = overflowArticle(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(article, { title: "T", body: "long body" }, labels);
  applyExpandState(article, true, labels, { animate: true, motion: false });
  assert.equal(article.classList.contains("is-expanded"), true);
  assert.equal(article.classList.contains("is-body-opening"), false);
  assert.equal(body.style.maxHeight, "");
  assert.equal(body.style.height, "");
  applyExpandState(article, false, labels, { animate: true, motion: false });
  assert.equal(article.classList.contains("is-expanded"), false);
  assert.equal(article.classList.contains("is-body-closing"), false);
  assert.equal(body.style.maxHeight, "");
});

test("pending refine hides the extractive sentence until writing is shown", () => {
  const compact = "The persist timeout is the real bug in";
  assert.deepEqual(
    titleSlotPresentation({
      phase: "pending",
      title: compact,
      elapsedMs: 0,
      motion: true,
      reveal: "enter",
    }),
    {
      pending: true,
      showTitle: false,
      showWriting: false,
      showSpinner: false,
      title: "",
      fade: false,
    },
  );
  assert.deepEqual(
    titleSlotPresentation({
      phase: "pending",
      title: compact,
      elapsedMs: TITLE_WRITING_DELAY_MS,
      motion: true,
      reveal: "enter",
    }),
    {
      pending: true,
      showTitle: false,
      showWriting: true,
      showSpinner: true,
      title: "",
      fade: false,
    },
  );
  const doc = createDocument();
  const article = writingArticleFixture(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(
    article,
    { title: compact, body: "body", titlePhase: "pending" },
    labels,
    { reveal: "enter", motion: true, now: 0 },
  );
  const title = article.querySelector("[data-slot=title]");
  const text = article.querySelector("[data-slot=title-text]");
  const writing = article.querySelector("[data-slot=title-writing]");
  const spinner = article.querySelector("[data-slot=title-spinner]");
  assert.equal(title.hidden, false);
  assert.equal(text.textContent, "");
  assert.doesNotMatch(title.textContent, /persist timeout/);
  assert.equal(article.classList.contains("is-title-pending"), true);
  assert.equal(writing.getAttribute("aria-hidden"), "true");
  assert.equal(spinner.hidden, true);
  assert.equal(title.getAttribute("aria-busy"), "true");
  assert.equal(title.getAttribute("aria-label"), "Writing title…");
  fillItemChrome(
    article,
    { title: compact, body: "body", titlePhase: "pending" },
    labels,
    { reveal: "enter", motion: true, now: TITLE_WRITING_DELAY_MS },
  );
  assert.equal(text.textContent, "");
  assert.equal(writing.getAttribute("aria-hidden"), "false");
  assert.equal(spinner.hidden, false);
  assert.equal(
    article.querySelector("[data-slot=title-writing-label]").textContent,
    "Writing title…",
  );
  assert.equal(title.getAttribute("aria-label"), null);
});

test("final title appears in the reserved slot after refine or fallback", () => {
  const generated = "Migration timeout";
  const fallback = "The persist timeout is the real bug in";
  assert.equal(
    titleSlotPresentation({
      phase: "final",
      title: generated,
      motion: true,
      reveal: "update",
    }).showTitle,
    true,
  );
  const doc = createDocument();
  const article = writingArticleFixture(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(
    article,
    { title: fallback, body: "body", titlePhase: "pending" },
    labels,
    { reveal: "enter", motion: true, now: TITLE_WRITING_DELAY_MS },
  );
  fillItemChrome(
    article,
    { title: generated, body: "body", titlePhase: "final" },
    labels,
    { reveal: "update", motion: true, now: 240 },
  );
  const title = article.querySelector("[data-slot=title]");
  assert.equal(title.hidden, false);
  assert.equal(
    article.querySelector("[data-slot=title-text]").textContent,
    generated,
  );
  assert.equal(article.classList.contains("is-title-pending"), false);
  assert.equal(article.classList.contains("is-title-revealing"), true);
  assert.equal(title.getAttribute("aria-busy"), null);
  fillItemChrome(
    article,
    { title: fallback, body: "body", titlePhase: "final" },
    labels,
    { reveal: "update", motion: true, now: 400 },
  );
  assert.equal(
    article.querySelector("[data-slot=title-text]").textContent,
    fallback,
  );
});

test("extractive titles never enter the writing-title state", () => {
  const extractive = "The extractive sentence is the title.";
  assert.equal(
    titleSlotPresentation({
      phase: "final",
      title: extractive,
      motion: true,
      reveal: "enter",
    }).showWriting,
    false,
  );
  const doc = createDocument();
  const article = writingArticleFixture(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(
    article,
    { title: extractive, body: "body", titlePhase: "final" },
    labels,
    { reveal: "enter", motion: true, now: 0 },
  );
  assert.equal(article.classList.contains("is-title-pending"), false);
  assert.equal(
    article.querySelector("[data-slot=title-text]").textContent,
    extractive,
  );
  assert.equal(article.querySelector("[data-slot=title-spinner]").hidden, true);
  assert.equal(
    article
      .querySelector("[data-slot=title-writing]")
      .getAttribute("aria-hidden"),
    "true",
  );
});

test("reduced motion shows static writing text and the final title immediately", () => {
  const compact = "Thanks for the note.";
  assert.deepEqual(
    titleSlotPresentation({
      phase: "pending",
      title: compact,
      elapsedMs: 0,
      motion: false,
      reveal: "enter",
    }),
    {
      pending: true,
      showTitle: false,
      showWriting: true,
      showSpinner: false,
      title: "",
      fade: false,
    },
  );
  const doc = createDocument();
  const article = writingArticleFixture(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(
    article,
    { title: compact, body: "body", titlePhase: "pending" },
    labels,
    { reveal: "enter", motion: false, now: 0 },
  );
  assert.equal(article.querySelector("[data-slot=title-text]").textContent, "");
  assert.equal(
    article.querySelector("[data-slot=title-writing-label]").textContent,
    "Writing title…",
  );
  assert.equal(article.querySelector("[data-slot=title-spinner]").hidden, true);
  assert.equal(article.classList.contains("is-title-revealing"), false);
  fillItemChrome(
    article,
    { title: "Migration timeout", body: "body", titlePhase: "final" },
    labels,
    { reveal: "update", motion: false, now: 20 },
  );
  assert.equal(
    article.querySelector("[data-slot=title-text]").textContent,
    "Migration timeout",
  );
  assert.equal(article.classList.contains("is-title-revealing"), false);
});

test("source icon accepts only rust png data urls", () => {
  assert.equal(
    sourceIconSrc("data:image/png;base64,abc"),
    "data:image/png;base64,abc",
  );
  assert.equal(sourceIconSrc("https://example.com/cursor.png"), null);
  assert.equal(sourceIconSrc("data:image/png;base64,http://evil"), null);
  assert.equal(sourceIconSrc("data:image/svg+xml;base64,abc"), null);
  assert.equal(sourceIconSrc(""), null);
});

test("source row shows catalog name and optional official icon", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  applySourceRow(
    article,
    formatCaptureSource("From {appName}", "Cursor"),
    "data:image/png;base64,abc",
  );
  const source = article.querySelector("[data-slot=source]");
  const icon = article.querySelector("[data-slot=source-icon]");
  assert.equal(source.hidden, false);
  assert.equal(
    source.querySelector("[data-slot=source-label]").textContent,
    "From Cursor",
  );
  assert.equal(icon.hidden, false);
  assert.equal(icon.src, "data:image/png;base64,abc");
  applySourceRow(article, "From Ghostty", "https://cdn.example/icon.png");
  assert.equal(icon.hidden, true);
  assert.equal(icon.src, "");
  applySourceRow(article, null, "data:image/png;base64,abc");
  assert.equal(source.hidden, true);
});

test("source row stays after a later title refine and a later insert patch", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  const labels = readExpandLabels(article);
  const icon = "data:image/png;base64,abc";
  fillItemChrome(
    article,
    {
      id: "whatsapp",
      title: "Aftrekker en dwijl",
      body: "Aftrekker en dwijl",
      sourceAppName: "WhatsApp",
      sourceAppIcon: icon,
    },
    labels,
  );
  const source = article.querySelector("[data-slot=source]");
  const img = article.querySelector("[data-slot=source-icon]");
  assert.equal(source.hidden, false);
  assert.equal(
    source.querySelector("[data-slot=source-label]").textContent,
    "From WhatsApp",
  );
  assert.equal(img.hidden, false);
  assert.equal(img.src, icon);
  fillItemChrome(
    article,
    {
      id: "whatsapp",
      title: "Aftrekker en dwijl",
      body: "Aftrekker en dwijl",
      sourceAppName: "WhatsApp",
      sourceAppIcon: icon,
    },
    labels,
  );
  assert.equal(source.hidden, false);
  assert.equal(
    source.querySelector("[data-slot=source-label]").textContent,
    "From WhatsApp",
  );
  assert.equal(img.src, icon);
  fillItemChrome(
    article,
    {
      id: "cursor",
      title: "Sort button on main",
      body: "later capture",
      sourceAppName: "Cursor",
      sourceAppIcon: icon,
    },
    labels,
  );
  assert.equal(source.hidden, false);
  assert.equal(
    source.querySelector("[data-slot=source-label]").textContent,
    "From Cursor",
  );
  assert.equal(img.hidden, false);
});

test("a patch that omits source keeps an existing source row", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  const labels = readExpandLabels(article);
  const icon = "data:image/png;base64,abc";
  applyItemSource(
    article,
    { sourceAppName: "WhatsApp", sourceAppIcon: icon },
    labels.sourceTemplate,
  );
  const source = article.querySelector("[data-slot=source]");
  const img = article.querySelector("[data-slot=source-icon]");
  fillItemChrome(
    article,
    { id: "whatsapp", title: "Refined title", body: "same body" },
    labels,
  );
  assert.equal(source.hidden, false);
  assert.equal(
    source.querySelector("[data-slot=source-label]").textContent,
    "From WhatsApp",
  );
  assert.equal(img.hidden, false);
  assert.equal(img.src, icon);
});

test("composer-only items stay without a source row", () => {
  const doc = createDocument();
  const article = articleFixture(doc);
  const labels = readExpandLabels(article);
  fillItemChrome(
    article,
    {
      id: "typed",
      title: "Typed note",
      body: "from the composer",
      sourceAppName: null,
      sourceAppIcon: null,
    },
    labels,
  );
  const source = article.querySelector("[data-slot=source]");
  assert.equal(source.hidden, true);
  assert.equal(
    source.querySelector("[data-slot=source-label]").textContent,
    "From {appName}",
  );
  fillItemChrome(
    article,
    { id: "typed", title: "Typed note", body: "from the composer" },
    labels,
  );
  assert.equal(source.hidden, true);
});
