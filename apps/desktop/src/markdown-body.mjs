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

function unescapeLiteral(source) {
  let out = "";
  let index = 0;
  while (index < source.length) {
    if (isEscapedPair(source, index)) {
      out += source[index + 1];
      index += 2;
      continue;
    }
    out += source[index];
    index += 1;
  }
  return out;
}

function flushText(nodes, buffer) {
  if (buffer.value.length === 0) {
    return;
  }
  nodes.push({ type: "text", value: buffer.value });
  buffer.value = "";
}

// Unescaped * / ** / *** and ` are markup. Every other character, including
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

    if (source.startsWith("`", index)) {
      const close = findUnescaped(source, index + 1, "`");
      if (close > index + 1) {
        flushText(nodes, buffer);
        nodes.push({
          type: "code",
          children: [
            {
              type: "text",
              value: unescapeLiteral(source.slice(index + 1, close)),
            },
          ],
        });
        index = close + 1;
        continue;
      }
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
    if (node.type === "code") {
      const code = doc.createElement("code");
      for (const child of astToNodes(doc, node.children)) {
        appendChild(code, child);
      }
      nodes.push(code);
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

function isAnyWhitespace(ch) {
  return /\s/u.test(ch);
}

function isMarkerGap(ch) {
  return (
    isAnyWhitespace(ch) &&
    ch !== "\n" &&
    ch !== "\r" &&
    ch !== "\u2028" &&
    ch !== "\u2029"
  );
}

function isLowerLetter(ch) {
  return ch !== ch.toUpperCase() && ch === ch.toLowerCase();
}

function isUpperLetter(ch) {
  return ch !== ch.toLowerCase() && ch === ch.toUpperCase();
}

function gluedListMarker(chars, index) {
  if (
    index === 0 ||
    isAnyWhitespace(chars[index - 1]) ||
    chars[index - 1] === "*"
  ) {
    return 0;
  }
  let end = index;
  if (end >= chars.length || chars[end] < "0" || chars[end] > "9") {
    return 0;
  }
  while (end < chars.length && chars[end] >= "0" && chars[end] <= "9") {
    end += 1;
  }
  if (end >= chars.length || chars[end] !== ".") {
    return 0;
  }
  end += 1;
  if (end >= chars.length || !isMarkerGap(chars[end])) {
    return 0;
  }
  while (end < chars.length && isMarkerGap(chars[end])) {
    end += 1;
  }
  return end - index;
}

function jammedSentence(chars, index) {
  if (chars[index] !== "." || index < 2) {
    return false;
  }
  if (!isLowerLetter(chars[index - 1]) || !isLowerLetter(chars[index - 2])) {
    return false;
  }
  const next = chars[index + 1];
  return Boolean(next) && isUpperLetter(next);
}

// Accessibility selections often omit block separators. A list marker glued
// to the previous word, or a sentence end jammed onto the next capital,
// is restored. Spaced prose is left as captured.
export function restoreSmashedStructure(markdown) {
  const source = typeof markdown === "string" ? markdown : "";
  const chars = Array.from(source);
  let out = "";
  let index = 0;
  while (index < chars.length) {
    const marker = gluedListMarker(chars, index);
    if (marker > 0) {
      out += "\n";
      out += chars.slice(index, index + marker).join("");
      index += marker;
      continue;
    }
    if (jammedSentence(chars, index)) {
      out += ".\n\n";
      index += 1;
      continue;
    }
    out += chars[index];
    index += 1;
  }
  return out;
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
    items: split.items.map((item) => ({
      value: null,
      heading: false,
      children: parseConstrainedMarkdown(item),
      blocks: [],
    })),
  });
}

function depthFromIndent(indent) {
  if (indent < 2) {
    return 0;
  }
  return Math.max(1, Math.floor(indent / 4));
}

function leadingIndent(raw) {
  let spaces = 0;
  let bytes = 0;
  for (const ch of raw) {
    if (ch === " ") {
      spaces += 1;
      bytes += 1;
    } else if (ch === "\t") {
      spaces += 4;
      bytes += 1;
    } else {
      break;
    }
  }
  return { spaces, bytes };
}

function stripWrappingBold(s) {
  if (!s.startsWith("**") || !s.endsWith("**") || s.length < 4) {
    return null;
  }
  const inner = s.slice(2, -2);
  if (inner.length === 0 || inner.includes("**")) {
    return null;
  }
  return inner;
}

function numberedMarker(s) {
  let digits = 0;
  while (digits < s.length && s[digits] >= "0" && s[digits] <= "9") {
    digits += 1;
  }
  if (digits === 0 || digits > 9 || s[digits] !== ".") {
    return null;
  }
  if (!isMarkerGap(s[digits + 1] ?? "")) {
    return null;
  }
  return {
    n: Number(s.slice(0, digits)),
    text: s.slice(digits + 1).trimStart(),
  };
}

function bulletMarker(s) {
  for (const prefix of ["- ", "+ ", "• ", "* "]) {
    if (s.startsWith(prefix)) {
      return s.slice(prefix.length);
    }
  }
  return null;
}

function classifyLine(rawLine) {
  const raw = rawLine.endsWith("\r") ? rawLine.slice(0, -1) : rawLine;
  const { spaces, bytes } = leadingIndent(raw);
  const rest = raw.slice(bytes);
  if (rest.trim().length === 0) {
    return { raw, indent: spaces, kind: "blank", depth: 0, heading: false };
  }
  const trimmed = rest.trim();
  const boldInner = stripWrappingBold(trimmed);
  if (boldInner) {
    const numbered = numberedMarker(boldInner);
    if (numbered) {
      return {
        raw,
        indent: spaces,
        kind: "ol",
        n: numbered.n,
        text: numbered.text,
        sourceBold: true,
        depth: 0,
        heading: false,
      };
    }
  }
  const numbered = numberedMarker(trimmed);
  if (numbered) {
    return {
      raw,
      indent: spaces,
      kind: "ol",
      n: numbered.n,
      text: numbered.text,
      sourceBold: false,
      depth: 0,
      heading: false,
    };
  }
  const bullet = bulletMarker(trimmed);
  if (bullet != null) {
    return {
      raw,
      indent: spaces,
      kind: "ul",
      text: bullet,
      depth: 0,
      heading: false,
    };
  }
  return { raw, indent: spaces, kind: "prose", depth: 0, heading: false };
}

function pushNumber(stack, n) {
  if (stack.length === 0) {
    stack.push(n);
    return 0;
  }
  if (n > stack[stack.length - 1]) {
    stack[stack.length - 1] = n;
    return stack.length - 1;
  }
  if (n === 1) {
    stack.push(1);
    return stack.length - 1;
  }
  while (stack.length > 1) {
    stack.pop();
    if (n > stack[stack.length - 1]) {
      stack[stack.length - 1] = n;
      return stack.length - 1;
    }
    if (n === 1) {
      stack.push(1);
      return stack.length - 1;
    }
  }
  stack[0] = n;
  return 0;
}

function markNumberedHeadings(lines) {
  const olIdx = [];
  for (let i = 0; i < lines.length; i += 1) {
    if (lines[i].kind === "ol") {
      olIdx.push(i);
    }
  }
  for (let pos = 0; pos < olIdx.length; pos += 1) {
    const i = olIdx[pos];
    if (lines[i].sourceBold) {
      lines[i].heading = true;
    }
    const depth = lines[i].depth;
    let hasChild = false;
    for (let next = pos + 1; next < olIdx.length; next += 1) {
      const j = olIdx[next];
      if (lines[j].depth > depth) {
        hasChild = true;
        break;
      }
      if (lines[j].depth <= depth) {
        break;
      }
    }
    if (hasChild) {
      lines[i].heading = true;
      continue;
    }
    let prev = -1;
    for (let earlier = pos - 1; earlier >= 0; earlier -= 1) {
      const j = olIdx[earlier];
      if (lines[j].depth === depth) {
        prev = j;
        break;
      }
    }
    if (prev < 0 || !lines[prev].heading) {
      continue;
    }
    let shallower = false;
    for (let earlier = 0; earlier < pos; earlier += 1) {
      const j = olIdx[earlier];
      if (j > prev && lines[j].depth < depth) {
        shallower = true;
        break;
      }
    }
    if (!shallower && lines[i].n === lines[prev].n + 1) {
      lines[i].heading = true;
    }
  }
}

function liftFlushUnderBold(lines) {
  let open = null;
  for (const line of lines) {
    if (line.kind === "blank") {
      continue;
    }
    if (line.kind === "ol" && line.sourceBold) {
      open = line.depth;
      line.heading = true;
      continue;
    }
    if (open == null || line.depth > open) {
      continue;
    }
    line.depth = open + 1;
    line.heading = false;
  }
}

function attachFollowers(lines) {
  let open = null;
  for (const line of lines) {
    if (line.kind === "blank") {
      continue;
    }
    if (line.kind === "ol" && line.heading) {
      open = line.depth;
      continue;
    }
    if (line.kind === "ol") {
      if (open != null && line.depth <= open) {
        open = null;
      }
      continue;
    }
    if (open != null && line.depth <= open) {
      line.depth = open + 1;
    }
  }
}

function assignOutline(lines) {
  const sourceLists = lines.some(
    (line) => line.indent >= 2 && (line.kind === "ol" || line.kind === "ul"),
  );
  const sourceBold = lines.some(
    (line) => line.kind === "ol" && line.sourceBold,
  );
  if (sourceLists) {
    for (const line of lines) {
      if (line.kind !== "blank") {
        line.depth = depthFromIndent(line.indent);
      }
    }
  } else {
    const stack = [];
    for (const line of lines) {
      if (line.kind === "ol") {
        line.depth = pushNumber(stack, line.n);
      }
    }
  }
  if (sourceBold) {
    liftFlushUnderBold(lines);
  }
  markNumberedHeadings(lines);
  attachFollowers(lines);
}

function proseText(line) {
  return line.depth === 0 ? line.raw : line.raw.trim();
}

function parseBlocks(lines, cursor, minDepth) {
  const blocks = [];
  let i = cursor.index;
  while (i < lines.length) {
    if (lines[i].kind === "blank") {
      if (minDepth === 0) {
        blocks.push({ type: "prose", text: "" });
        i += 1;
        continue;
      }
      let k = i;
      while (k < lines.length && lines[k].kind === "blank") {
        k += 1;
      }
      if (k >= lines.length || lines[k].depth < minDepth) {
        break;
      }
      blocks.push({ type: "prose", text: "" });
      i += 1;
      continue;
    }
    if (lines[i].depth < minDepth) {
      break;
    }
    if (lines[i].kind === "ol" || lines[i].kind === "ul") {
      const depth = lines[i].depth;
      const ordered = lines[i].kind === "ol";
      const items = [];
      while (i < lines.length) {
        const cur = lines[i];
        const sameKind = ordered ? cur.kind === "ol" : cur.kind === "ul";
        if (cur.depth !== depth || !sameKind || cur.kind === "blank") {
          break;
        }
        const text = cur.text ?? "";
        const heading = Boolean(cur.heading);
        const value = cur.kind === "ol" ? String(cur.n) : null;
        i += 1;
        const children = parseBlocks(lines, { index: i }, depth + 1);
        i = children.next;
        items.push({ value, heading, text, blocks: children.blocks });
      }
      blocks.push({ type: ordered ? "ol" : "ul", items });
      continue;
    }
    blocks.push({ type: "prose", text: proseText(lines[i]) });
    i += 1;
  }
  cursor.index = i;
  return { blocks, next: i };
}

function finalizeList(block) {
  return {
    type: block.type,
    items: block.items.map((item) => ({
      value: item.value,
      heading: item.heading,
      children: parseConstrainedMarkdown(item.text),
      blocks: finalize(item.blocks, true),
    })),
  };
}

function finalize(blocks, nested) {
  if (!nested) {
    const out = [];
    let buf = [];
    const flush = () => {
      if (buf.length === 0) {
        return;
      }
      pushParagraph(out, buf.join("\n"));
      buf = [];
    };
    for (const block of blocks) {
      if (block.type === "prose") {
        buf.push(block.text);
      } else {
        flush();
        out.push(finalizeList(block));
      }
    }
    flush();
    return out;
  }
  const out = [];
  for (const block of blocks) {
    if (block.type === "prose") {
      if (block.text.trim().length === 0) {
        continue;
      }
      out.push({
        type: "paragraph",
        children: parseConstrainedMarkdown(block.text.trim()),
      });
      continue;
    }
    out.push(finalizeList(block));
  }
  return out;
}

export function parseConstrainedDocument(markdown) {
  const source = restoreSmashedStructure(markdown);
  const lines = source.split("\n").map(classifyLine);
  assignOutline(lines);
  const parsed = parseBlocks(lines, { index: 0 }, 0);
  return finalize(parsed.blocks, false);
}

function documentToNodes(doc, blocks, nested) {
  const nodes = [];
  for (const block of blocks) {
    if (block.type === "paragraph") {
      const paragraph = doc.createElement("p");
      if (nested) {
        paragraph.setAttribute(
          "style",
          "margin:0;padding-inline-start:1.25em;font-weight:400",
        );
      }
      for (const child of astToNodes(doc, block.children)) {
        appendChild(paragraph, child);
      }
      nodes.push(paragraph);
      continue;
    }
    const list = doc.createElement(block.type === "ol" ? "ol" : "ul");
    if (nested) {
      list.setAttribute("style", "padding-inline-start:1.25em;font-weight:400");
    }
    for (const item of block.items) {
      const li = doc.createElement("li");
      if (block.type === "ol" && item.value != null) {
        li.setAttribute("value", item.value);
      }
      if (item.heading) {
        li.setAttribute("style", "font-weight:650");
        const strong = doc.createElement("strong");
        for (const child of astToNodes(doc, item.children)) {
          appendChild(strong, child);
        }
        appendChild(li, strong);
      } else {
        for (const child of astToNodes(doc, item.children)) {
          appendChild(li, child);
        }
      }
      for (const child of documentToNodes(doc, item.blocks ?? [], true)) {
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
  const nodes = documentToNodes(doc, parseConstrainedDocument(markdown), false);
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
  if (tag === "code") {
    return inner ? `\`${inner}\`` : "";
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
