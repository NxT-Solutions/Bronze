const ESCAPABLE = new Set(["\\", "*", "`"]);

function isEscapedPair(source, index) {
  return (
    source[index] === "\\" &&
    index + 1 < source.length &&
    ESCAPABLE.has(source[index + 1])
  );
}

function findUnescaped(source, start, marker) {
  let index = start;
  while (index < source.length) {
    if (isEscapedPair(source, index)) {
      index += 2;
      continue;
    }
    if (source.startsWith(marker, index)) {
      return index;
    }
    index += 1;
  }
  return -1;
}

function flushText(nodes, buffer) {
  if (buffer.value.length === 0) {
    return;
  }
  nodes.push({ type: "text", value: buffer.value });
  buffer.value = "";
}

// Only unescaped * / ** / *** are markup. Every other character, including
// HTML tags and attributes, stays a text node.
export function parseConstrainedMarkdown(markdown) {
  const source = typeof markdown === "string" ? markdown : "";
  const nodes = [];
  const buffer = { value: "" };
  let index = 0;

  while (index < source.length) {
    if (isEscapedPair(source, index)) {
      buffer.value += source[index + 1];
      index += 2;
      continue;
    }

    let matched = false;
    for (const marker of ["***", "**", "*"]) {
      if (!source.startsWith(marker, index)) {
        continue;
      }
      const close = findUnescaped(source, index + marker.length, marker);
      if (close === -1) {
        continue;
      }
      flushText(nodes, buffer);
      const inner = source.slice(index + marker.length, close);
      nodes.push({
        type: marker === "***" ? "strongEm" : marker === "**" ? "strong" : "em",
        children: parseConstrainedMarkdown(inner),
      });
      index = close + marker.length;
      matched = true;
      break;
    }
    if (matched) {
      continue;
    }

    buffer.value += source[index];
    index += 1;
  }

  flushText(nodes, buffer);
  return nodes;
}

function appendChild(parent, child) {
  parent.appendChild(child);
}

function astToNodes(doc, ast) {
  const nodes = [];
  for (const node of ast) {
    if (node.type === "text") {
      nodes.push(doc.createTextNode(node.value));
      continue;
    }
    if (node.type === "strong") {
      const strong = doc.createElement("strong");
      for (const child of astToNodes(doc, node.children)) {
        appendChild(strong, child);
      }
      nodes.push(strong);
      continue;
    }
    if (node.type === "em") {
      const em = doc.createElement("em");
      for (const child of astToNodes(doc, node.children)) {
        appendChild(em, child);
      }
      nodes.push(em);
      continue;
    }
    const strong = doc.createElement("strong");
    const em = doc.createElement("em");
    for (const child of astToNodes(doc, node.children)) {
      appendChild(em, child);
    }
    appendChild(strong, em);
    nodes.push(strong);
  }
  return nodes;
}

const UL_ITEM = /^(?:[ \t]*)(?:[-+•]|\*(?= ))[ \t]+(.*)$/;
const OL_ITEM = /^(?:[ \t]*)\d+\.[ \t]+(.*)$/;

function matchListRun(lines, start, pattern) {
  if (!pattern.test(lines[start] ?? "")) {
    return null;
  }
  const items = [];
  let index = start;
  while (index < lines.length) {
    const match = lines[index].match(pattern);
    if (!match) {
      break;
    }
    items.push(match[1]);
    index += 1;
  }
  return items.length > 0 ? { items, next: index } : null;
}

function splitInlineBullets(text) {
  if (!text.includes("•")) {
    return null;
  }
  const parts = text
    .split("•")
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  if (parts.length < 2) {
    return null;
  }
  return { lead: parts[0], items: parts.slice(1) };
}

function pushParagraph(blocks, text) {
  const split = splitInlineBullets(text);
  if (!split) {
    blocks.push({
      type: "paragraph",
      children: parseConstrainedMarkdown(text),
    });
    return;
  }
  if (split.lead) {
    blocks.push({
      type: "paragraph",
      children: parseConstrainedMarkdown(split.lead),
    });
  }
  blocks.push({
    type: "ul",
    items: split.items.map((item) => parseConstrainedMarkdown(item)),
  });
}

export function parseConstrainedDocument(markdown) {
  const source = typeof markdown === "string" ? markdown : "";
  const lines = source.split(/\r?\n/);
  const blocks = [];
  let index = 0;
  while (index < lines.length) {
    const ul = matchListRun(lines, index, UL_ITEM);
    if (ul) {
      blocks.push({
        type: "ul",
        items: ul.items.map((item) => parseConstrainedMarkdown(item)),
      });
      index = ul.next;
      continue;
    }
    const ol = matchListRun(lines, index, OL_ITEM);
    if (ol) {
      blocks.push({
        type: "ol",
        items: ol.items.map((item) => parseConstrainedMarkdown(item)),
      });
      index = ol.next;
      continue;
    }
    const para = [];
    while (
      index < lines.length &&
      !UL_ITEM.test(lines[index]) &&
      !OL_ITEM.test(lines[index])
    ) {
      para.push(lines[index]);
      index += 1;
    }
    pushParagraph(blocks, para.join("\n"));
  }
  return blocks;
}

function documentToNodes(doc, blocks) {
  const nodes = [];
  for (const block of blocks) {
    if (block.type === "paragraph") {
      const paragraph = doc.createElement("p");
      for (const child of astToNodes(doc, block.children)) {
        appendChild(paragraph, child);
      }
      nodes.push(paragraph);
      continue;
    }
    const list = doc.createElement(block.type === "ol" ? "ol" : "ul");
    for (const item of block.items) {
      const li = doc.createElement("li");
      for (const child of astToNodes(doc, item)) {
        appendChild(li, child);
      }
      appendChild(list, li);
    }
    nodes.push(list);
  }
  return nodes;
}

export function renderMarkdownBody(targetEl, markdown) {
  if (!targetEl) {
    return;
  }
  const doc = targetEl.ownerDocument;
  if (!doc?.createElement || !doc?.createTextNode) {
    throw new Error("MarkdownBodyDocumentUnavailable");
  }
  const nodes = documentToNodes(doc, parseConstrainedDocument(markdown));
  targetEl.replaceChildren(...nodes);
}

function nodeName(node) {
  return String(node.tagName || node.nodeName || "").toLowerCase();
}

function childrenOf(node) {
  if (node.childNodes && typeof node.childNodes.length === "number") {
    return Array.from(node.childNodes);
  }
  if (Array.isArray(node.children)) {
    return node.children;
  }
  return [];
}

function escapeMarkdownText(value) {
  let out = "";
  for (const ch of String(value)) {
    if (ch === "\u200B") {
      continue;
    }
    if (ESCAPABLE.has(ch)) {
      out += `\\${ch}`;
    } else {
      out += ch;
    }
  }
  return out;
}

function serializeInline(node) {
  if (node.nodeType === 3 || node.nodeName === "#text") {
    return escapeMarkdownText(node.data ?? node.textContent ?? "");
  }
  if (node.nodeType !== 1) {
    return "";
  }
  const tag = nodeName(node);
  if (tag === "script" || tag === "style" || tag === "noscript") {
    return "";
  }
  if (tag === "br") {
    return "\n";
  }
  const inner = childrenOf(node).map(serializeInline).join("");
  if (tag === "strong" || tag === "b") {
    return inner ? `**${inner}**` : "";
  }
  if (tag === "em" || tag === "i") {
    return inner ? `*${inner}*` : "";
  }
  return inner;
}

function serializeList(node) {
  const ordered = nodeName(node) === "ol";
  return childrenOf(node)
    .filter((child) => nodeName(child) === "li")
    .map((item, index) => {
      const text = childrenOf(item).map(serializeInline).join("");
      return ordered ? `${index + 1}. ${text}` : `- ${text}`;
    })
    .join("\n");
}

function serializeBlocks(node) {
  const parts = [];
  let inline = "";

  function flushInline() {
    if (inline.length > 0) {
      parts.push(inline);
      inline = "";
    }
  }

  for (const child of childrenOf(node)) {
    if (child.nodeType === 3 || child.nodeName === "#text") {
      inline += escapeMarkdownText(child.data ?? child.textContent ?? "");
      continue;
    }
    if (child.nodeType !== 1) {
      continue;
    }
    const tag = nodeName(child);
    if (tag === "script" || tag === "style" || tag === "noscript") {
      continue;
    }
    if (tag === "br") {
      flushInline();
      continue;
    }
    if (tag === "ul" || tag === "ol") {
      flushInline();
      const list = serializeList(child);
      if (list) {
        parts.push(list);
      }
      continue;
    }
    if (tag === "div" || tag === "p") {
      flushInline();
      const line = serializeBlocks(child);
      if (line.length > 0) {
        parts.push(line);
      }
      continue;
    }
    inline += serializeInline(child);
  }
  flushInline();
  return parts.join("\n");
}

export function serializeComposerDom(root) {
  if (!root) {
    return "";
  }
  return serializeBlocks(root).replace(/\n+$/u, "");
}
