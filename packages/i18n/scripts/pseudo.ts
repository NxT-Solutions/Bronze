#!/usr/bin/env tsx
// packages/i18n/scripts/pseudo.ts
// Generates en-XA and ar-XB from en catalog. Pure deterministic.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const EN = resolve(ROOT, "locales/en/app.json");
const EN_XA = resolve(ROOT, "locales/en-XA/app.json");
const AR_XB = resolve(ROOT, "locales/ar-XB/app.json");

function accent(c: string): string {
  // combining accents for en-XA, expand ~140%
  const map: Record<string, string> = {
    a: "á",
    e: "é",
    i: "í",
    o: "ó",
    u: "ú",
    A: "Á",
    E: "É",
    I: "Í",
    O: "Ó",
    U: "Ú",
    c: "ç",
    n: "ñ",
  };
  return map[c] || c;
}

function protect(value: string): { protected: string; parts: string[] } {
  const parts: string[] = [];
  let res = "";
  let i = 0;
  const n = value.length;
  while (i < n) {
    if (value[i] === "{") {
      let depth = 0;
      let j = i;
      while (j < n) {
        if (value[j] === "{") depth++;
        else if (value[j] === "}") depth--;
        j++;
        if (depth === 0) break;
      }
      const block = value.slice(i, j);
      const idx = parts.length;
      parts.push(block);
      res += `\uE000${idx}\uE001`;
      i = j;
      continue;
    }
    res += value[i];
    i++;
  }
  return { protected: res, parts };
}

function restore(s: string, parts: string[], wrapSimple = false): string {
  return s.replace(/\uE000(\d+)\uE001/g, (_, i) => {
    const ph = parts[Number(i)] || "";
    if (wrapSimple && !ph.includes(",")) {
      return `\u2067${ph}\u2069`;
    }
    return ph;
  });
}

export function pseudoEnXA(value: string): string {
  const { protected: prot_s, parts } = protect(value);
  let accented = "";
  for (const ch of prot_s) {
    if (/\p{L}/u.test(ch)) {
      accented += accent(ch) + ch;
    } else {
      accented += ch;
    }
  }
  const target = Math.max(value.length + 4, Math.ceil(value.length * 1.4));
  let padded = accented;
  const pad = " [·] ";
  while (padded.length < target) padded += pad;
  const restored = restore(padded, parts);
  return `【${restored}】`;
}

export function pseudoArXB(value: string): string {
  const { protected: prot_s, parts } = protect(value);
  const partsSplit = prot_s.split(/(\uE000\d+\uE001)/);
  for (let ii = 0; ii < partsSplit.length; ii++) {
    if (!partsSplit[ii].startsWith("\uE000")) {
      const s = partsSplit[ii];
      // reverse chars (non-space groups) for bidi stress test; keep ws order reversed too
      partsSplit[ii] = s
        .split(/(\s+)/)
        .map((t) => (t.trim() ? t.split("").reverse().join("") : t))
        .reverse()
        .join("");
    }
  }
  return restore(partsSplit.join(""), parts, true);
}

function main() {
  const en = JSON.parse(readFileSync(EN, "utf8"));
  const xa: Record<string, string> = {};
  const xb: Record<string, string> = {};
  for (const [k, v] of Object.entries(en)) {
    if (typeof v !== "string") continue;
    xa[k] = pseudoEnXA(v);
    xb[k] = pseudoArXB(v);
  }
  mkdirSync(dirname(EN_XA), { recursive: true });
  mkdirSync(dirname(AR_XB), { recursive: true });
  writeFileSync(EN_XA, `${JSON.stringify(xa, null, 2)}\n`);
  writeFileSync(AR_XB, `${JSON.stringify(xb, null, 2)}\n`);
  console.log("pseudo generated en-XA, ar-XB");
}

function isCli(): boolean {
  const entry = process.argv[1];
  if (!entry) return false;
  try {
    return import.meta.url === pathToFileURL(resolve(entry)).href;
  } catch {
    return false;
  }
}

if (isCli()) {
  main();
}
