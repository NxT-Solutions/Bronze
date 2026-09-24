import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  parseConstrainedDocument,
  parseConstrainedMarkdown,
  renderMarkdownBody,
  restoreSmashedStructure,
  serializeComposerDom,
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
  assert.deepEqual(tagsUnder(target), ["p", "strong"]);
  assert.equal(target.querySelector("strong").textContent, "Hello");
  assert.equal(target.querySelector("script"), null);
  assert.equal(target.querySelector("img"), null);

  renderMarkdownBody(target, "<script>alert(1)</script>");
  assert.equal(target.textContent, "<script>alert(1)</script>");
  assert.deepEqual(tagsUnder(target), ["p"]);
  assert.equal(target.querySelector("script"), null);

  renderMarkdownBody(target, '<img onerror="alert(1)">');
  assert.equal(target.textContent, '<img onerror="alert(1)">');
  assert.deepEqual(tagsUnder(target), ["p"]);
  assert.equal(target.querySelector("img"), null);

  renderMarkdownBody(target, "**Hello** <script>alert(1)</script>");
  assert.deepEqual(tagsUnder(target), ["p", "strong"]);
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
  const numbered = target.querySelectorAll("li");
  assert.equal(numbered[0].getAttribute("value"), "1");
  assert.equal(numbered[1].getAttribute("value"), "2");
  renderMarkdownBody(target, "- <script>alert(1)</script>");
  assert.equal(target.querySelector("script"), null);
  assert.match(target.textContent, /<script>alert\(1\)<\/script>/);
  renderMarkdownBody(
    target,
    "today's versions:• Follo Studio 0.2.21• The follo command 0.1.7",
  );
  assert.deepEqual(tagsUnder(target), ["p", "ul", "li", "li"]);
  assert.match(target.textContent, /Follo Studio 0.2.21/);
});

function textNode(value) {
  return { nodeType: 3, nodeName: "#text", data: value };
}

function el(tag, kids) {
  return {
    nodeType: 1,
    tagName: tag.toUpperCase(),
    nodeName: tag.toUpperCase(),
    childNodes: kids,
  };
}

test("composer DOM serializes to constrained markdown and drops scripts", () => {
  assert.equal(
    serializeComposerDom({
      childNodes: [el("strong", [textNode("Hello")]), textNode(" world")],
    }),
    "**Hello** world",
  );
  assert.equal(
    serializeComposerDom({
      childNodes: [el("em", [textNode("hi")])],
    }),
    "*hi*",
  );
  assert.equal(
    serializeComposerDom({
      childNodes: [
        el("div", [textNode("one")]),
        el("div", [el("strong", [textNode("two")])]),
      ],
    }),
    "one\n**two**",
  );
  assert.equal(
    serializeComposerDom({
      childNodes: [
        el("ul", [
          el("li", [textNode("a")]),
          el("li", [el("em", [textNode("b")])]),
        ]),
      ],
    }),
    "- a\n- *b*",
  );
  assert.equal(
    serializeComposerDom({
      childNodes: [el("br", [])],
    }),
    "",
  );
  assert.equal(
    serializeComposerDom({
      childNodes: [el("script", [textNode("alert(1)")]), textNode("*")],
    }),
    "\\*",
  );
  assert.equal(
    serializeComposerDom({
      childNodes: [el("strong", [textNode("\u200BHello")])],
    }),
    "**Hello**",
  );
});

test("smashed lists keep source numbers and jammed sentences break", () => {
  const smashed =
    "één tegelijk1. DataForSEO (klaar)1. Grant is)2. Disable prod4. Eén login6. Unpause snapshot2. Asana\nscheduler.3. OpenRouter\nNiet. Schedule blijft paused.Niet alle";
  const restored = restoreSmashedStructure(smashed);
  assert.match(restored, /tegelijk\n1\. DataForSEO/);
  assert.match(restored, /\(klaar\)\n1\. Grant/);
  assert.match(restored, /is\)\n2\. Disable/);
  assert.match(restored, /prod\n4\. Eén/);
  assert.match(restored, /login\n6\. Unpause/);
  assert.match(restored, /snapshot\n2\. Asana/);
  assert.match(restored, /scheduler\.\n3\. OpenRouter/);
  assert.match(restored, /paused\.\n\nNiet alle/);
  assert.equal(restoreSmashedStructure(restored), restored);
  assert.equal(
    restoreSmashedStructure("see section 2. Next stays."),
    "see section 2. Next stays.",
  );
  assert.equal(restoreSmashedStructure("Hello. World"), "Hello. World");
  assert.equal(
    restoreSmashedStructure("Mr.Smith e.g.The version 1.2"),
    "Mr.Smith e.g.The version 1.2",
  );

  const doc = createDocument();
  const target = doc.createElement("div");
  renderMarkdownBody(
    target,
    "één tegelijk1. DataForSEO (klaar)1. Grant is)2. Disable",
  );
  assert.deepEqual(tagsUnder(target), ["p", "ol", "li", "li", "li"]);
  const lis = target.querySelectorAll("li");
  assert.equal(lis[0].getAttribute("value"), "1");
  assert.equal(lis[0].textContent, "DataForSEO (klaar)");
  assert.equal(lis[1].getAttribute("value"), "1");
  assert.equal(lis[1].textContent, "Grant is)");
  assert.equal(lis[2].getAttribute("value"), "2");
  assert.equal(lis[2].textContent, "Disable");
  assert.equal(target.querySelector("script"), null);

  renderMarkdownBody(target, "Schedule blijft paused.Niet alle");
  assert.equal(target.textContent, "Schedule blijft paused.\n\nNiet alle");
  assert.equal(target.querySelector("ol"), null);

  renderMarkdownBody(target, "1. <script>alert(1)</script>");
  assert.equal(target.querySelector("script"), null);
  assert.match(target.textContent, /<script>alert\(1\)<\/script>/);
  assert.equal(target.querySelector("li").getAttribute("value"), "1");
});

test("missing document is unavailable rather than assigned as HTML", () => {
  assert.throws(
    () => renderMarkdownBody({ replaceChildren() {} }, "**x**"),
    /MarkdownBodyDocumentUnavailable/,
  );
});
