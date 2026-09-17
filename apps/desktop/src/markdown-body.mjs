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
    blocks.push({
      type: "paragraph",
      children: parseConstrainedMarkdown(para.join("\n")),
    });
  }
  return blocks;
}

function documentToNodes(doc, blocks) {
  const nodes = [];
  for (const block of blocks) {
    if (block.type === "paragraph") {
      nodes.push(...astToNodes(doc, block.children));
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
