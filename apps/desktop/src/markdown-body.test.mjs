import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  parseConstrainedDocument,
  parseConstrainedMarkdown,
  renderMarkdownBody,
} from "./markdown-body.mjs";

const root = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(join(root, "markdown-body.mjs"), "utf8");

function createDocument() {
  function createElement(tagName) {
    const children = [];
    const attrs = {};
    const el = {
      nodeType: 1,
      tagName: tagName.toUpperCase(),
      children,
      childNodes: children,
      hidden: false,
      dataset: {},
      className: "",
      ownerDocument: doc,
      appendChild(child) {
        children.push(child);
        return child;
      },
      replaceChildren(...nodes) {
        children.length = 0;
        children.push(...nodes);
      },
      setAttribute(name, value) {
        attrs[name] = String(value);
      },
      getAttribute(name) {
        return Object.hasOwn(attrs, name) ? attrs[name] : null;
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
        return children.map((child) => child.textContent ?? "").join("");
      },
      set(value) {
        children.length = 0;
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

function tagsUnder(el) {
  const tags = [];
  const stack = [...(el.children ?? [])];
  while (stack.length) {
    const node = stack.shift();
    if (node.nodeType === 1) {
      tags.push(node.tagName.toLowerCase());
      stack.unshift(...(node.children ?? []));
    }
  }
  return tags;
}

test("constrained markdown builds strong em and newlines without HTML", () => {
  assert.doesNotMatch(source, /innerHTML/);
  const ast = parseConstrainedMarkdown("**Hello**\n*world*");
  assert.deepEqual(ast, [
    { type: "strong", children: [{ type: "text", value: "Hello" }] },
    { type: "text", value: "\n" },
    { type: "em", children: [{ type: "text", value: "world" }] },
  ]);
  assert.deepEqual(parseConstrainedMarkdown("***both***"), [
    { type: "strongEm", children: [{ type: "text", value: "both" }] },
  ]);
  assert.deepEqual(
    parseConstrainedMarkdown("plain \\* star \\\\ and \\` tick"),
    [{ type: "text", value: "plain * star \\ and ` tick" }],
  );
});

test("renderMarkdownBody uses elements for emphasis and text for leaked HTML", () => {
  const doc = createDocument();
  const target = doc.createElement("div");
  renderMarkdownBody(target, "**Hello**");
  assert.deepEqual(tagsUnder(target), ["strong"]);
  assert.equal(target.querySelector("strong").textContent, "Hello");
  assert.equal(target.querySelector("script"), null);
  assert.equal(target.querySelector("img"), null);

  renderMarkdownBody(target, "<script>alert(1)</script>");
  assert.equal(target.textContent, "<script>alert(1)</script>");
  assert.deepEqual(tagsUnder(target), []);
  assert.equal(target.querySelector("script"), null);

  renderMarkdownBody(target, '<img onerror="alert(1)">');
  assert.equal(target.textContent, '<img onerror="alert(1)">');
  assert.deepEqual(tagsUnder(target), []);
  assert.equal(target.querySelector("img"), null);

  renderMarkdownBody(target, "**Hello** <script>alert(1)</script>");
  assert.deepEqual(tagsUnder(target), ["strong"]);
  assert.equal(target.textContent, "Hello <script>alert(1)</script>");
  assert.equal(target.querySelector("a"), null);
});

test("line-start markers become lists and leaked HTML stays text", () => {
  const doc = createDocument();
  const target = doc.createElement("div");
  const listed = parseConstrainedDocument("- one\n- **two**\n*italic*");
  assert.equal(listed[0].type, "ul");
  assert.equal(listed[0].items.length, 2);
  renderMarkdownBody(target, "- one\n- **two**");
  assert.deepEqual(tagsUnder(target), ["ul", "li", "li", "strong"]);
  assert.equal(target.querySelector("strong").textContent, "two");
  renderMarkdownBody(target, "1. first\n2. second");
  assert.deepEqual(tagsUnder(target), ["ol", "li", "li"]);
  renderMarkdownBody(target, "- <script>alert(1)</script>");
  assert.equal(target.querySelector("script"), null);
  assert.match(target.textContent, /<script>alert\(1\)<\/script>/);
  renderMarkdownBody(
    target,
    "today's versions:• Follo Studio 0.2.21• The follo command 0.1.7",
  );
  assert.deepEqual(tagsUnder(target), ["ul", "li", "li"]);
  assert.match(target.textContent, /Follo Studio 0.2.21/);
});

test("missing document is unavailable rather than assigned as HTML", () => {
  assert.throws(
    () => renderMarkdownBody({ replaceChildren() {} }, "**x**"),
    /MarkdownBodyDocumentUnavailable/,
  );
});
