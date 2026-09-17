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

export function renderMarkdownBody(targetEl, markdown) {
  if (!targetEl) {
    return;
  }
  const doc = targetEl.ownerDocument;
  if (!doc?.createElement || !doc?.createTextNode) {
    throw new Error("MarkdownBodyDocumentUnavailable");
  }
  const nodes = astToNodes(doc, parseConstrainedMarkdown(markdown));
  targetEl.replaceChildren(...nodes);
}
