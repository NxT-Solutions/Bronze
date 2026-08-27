#!/usr/bin/env tsx

import { readdirSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const LOCALES = resolve(ROOT, "locales");
const BASE = "en";

export function loadLocale(
  tag: string,
  localesDir = LOCALES,
): Record<string, string> {
  const p = resolve(localesDir, tag, "app.json");
  const raw = readFileSync(p, "utf8");
  const obj = JSON.parse(raw);
  if (!obj || typeof obj !== "object" || Array.isArray(obj)) {
    throw new Error(`${tag} catalog must be an object`);
  }
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(obj)) {
    if (typeof v !== "string") {
      throw new Error(`${tag}:${k} is not a string`);
    }
    out[k] = v;
  }
  return out;
}

function skipBalanced(msg: string, start: number): number {
  let depth = 0;
  let j = start;
  const n = msg.length;
  while (j < n) {
    if (msg[j] === "{") depth++;
    else if (msg[j] === "}") {
      depth--;
      j++;
      if (depth === 0) return j;
      continue;
    }
    j++;
  }
  return start + 1;
}

export function extractPlaceholders(msg: string): string[] {
  const names = new Set<string>();
  let i = 0;
  const n = msg.length;
  while (i < n) {
    if (msg[i] === "{") {
      const end = skipBalanced(msg, i);
      if (end === i + 1) {
        i++;
        continue;
      }
      let j = i + 1;
      while (j < end && /[A-Za-z0-9_]/.test(msg[j])) j++;
      const name = msg.slice(i + 1, j);
      if (name) names.add(name);
      i = end;
      continue;
    }
    i++;
  }
  return [...names].sort();
}

export function hasIcu(msg: string): boolean {
  return /,\s*(plural|select|selectordinal|number|date|time)\s*,/i.test(msg);
}

function stripBalancedBraces(msg: string): string {
  let out = "";
  let i = 0;
  const n = msg.length;
  while (i < n) {
    if (msg[i] === "{") {
      const end = skipBalanced(msg, i);
      if (end === i + 1) {
        out += msg[i];
        i++;
        continue;
      }
      i = end;
      continue;
    }
    out += msg[i];
    i++;
  }
  return out;
}

export function looksConcatenated(msg: string): boolean {
  if (/\s\+\s/.test(msg)) return true;
  return /[.!?]["']?\s+[A-Z]/.test(stripBalancedBraces(msg));
}

export function isValidBcp47Tag(tag: string): boolean {
  if (!/^[A-Za-z]{2,3}(-[A-Za-z0-9]{2,8})*$/.test(tag)) return false;
  try {
    const canon = Intl.getCanonicalLocales(tag.replace(/_/g, "-"));
    return canon.length > 0 && !canon[0].toLowerCase().startsWith("und");
  } catch {
    return false;
  }
}

export function collectCatalogErrors(
  catalogs: Record<string, Record<string, string>>,
  options: { base?: string; tags?: string[] } = {},
): string[] {
  const baseTag = options.base ?? BASE;
  const tags = options.tags ?? Object.keys(catalogs);
  const errors: string[] = [];
  const base = catalogs[baseTag];
  if (!base) {
    errors.push(`Cannot load ${baseTag}`);
    return errors;
  }
  const baseKeys = Object.keys(base).sort();

  for (const [k, v] of Object.entries(base)) {
    if (looksConcatenated(v)) {
      errors.push(`${baseTag}:${k} looks concatenated`);
    }
  }

  for (const tag of tags) {
    if (tag === baseTag) continue;
    const loc = catalogs[tag];
    if (!loc) {
      errors.push(`Cannot load ${tag}`);
      continue;
    }
    const locKeys = Object.keys(loc).sort();
    const missing = baseKeys.filter((k) => !locKeys.includes(k));
    const extra = locKeys.filter((k) => !baseKeys.includes(k));
    if (missing.length)
      errors.push(`${tag}: missing keys ${missing.join(",")}`);
    if (extra.length) errors.push(`${tag}: extra keys ${extra.join(",")}`);

    for (const k of baseKeys) {
      if (!Object.hasOwn(loc, k)) continue;
      const b = base[k];
      const l = loc[k];
      const bp = extractPlaceholders(b);
      const lp = extractPlaceholders(l);
      if (JSON.stringify(bp) !== JSON.stringify(lp)) {
        errors.push(`${tag}:${k} placeholder mismatch base=${bp} got=${lp}`);
      }
      if (hasIcu(b) && !hasIcu(l)) {
        errors.push(`${tag}:${k} ICU structure lost`);
      }
    }
  }

  return errors;
}

export function listLocaleTags(localesDir = LOCALES): string[] {
  return readdirSync(localesDir, { withFileTypes: true })
    .filter((d) => d.isDirectory() && !d.name.startsWith("."))
    .map((d) => d.name)
    .sort();
}

export function runValidate(localesDir = LOCALES): {
  ok: boolean;
  errors: string[];
} {
  const tags = listLocaleTags(localesDir);
  const errors: string[] = [];
  for (const t of tags) {
    if (!isValidBcp47Tag(t)) errors.push(`invalid tag ${t}`);
  }
  const catalogs: Record<string, Record<string, string>> = {};
  for (const t of tags) {
    try {
      catalogs[t] = loadLocale(t, localesDir);
    } catch (e) {
      errors.push(`Cannot load ${t}: ${e}`);
    }
  }
  if (catalogs[BASE]) {
    errors.push(...collectCatalogErrors(catalogs, { tags }));
  } else if (!errors.some((e) => e.includes(`Cannot load ${BASE}`))) {
    errors.push(`Cannot load ${BASE}`);
  }
  return { ok: errors.length === 0, errors };
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

function main(): void {
  const { ok, errors } = runValidate();
  if (!ok) {
    console.error("VALIDATE FAIL:");
    for (const e of errors) console.error(` - ${e}`);
    process.exit(1);
  }
}

if (isCli()) {
  main();
}
