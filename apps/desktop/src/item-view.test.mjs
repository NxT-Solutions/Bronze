import assert from "node:assert/strict";
import { test } from "node:test";
import {
  applyExpandState,
  applySourceRow,
  fillItemChrome,
  formatCaptureSource,
  readExpandLabels,
  sourceIconSrc,
  syncExpandVisibility,
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
      ownerDocument: doc,
      get classList() {
        return {
          contains: (name) => el.className.split(/\s+/).includes(name),
          add(name) {
            if (!this.contains(name)) {
              el.className = `${el.className} ${name}`.trim();
            }
          },
          remove(name) {
            el.className = el.className
              .split(/\s+/)
              .filter((part) => part && part !== name)
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

  const doc = {
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
