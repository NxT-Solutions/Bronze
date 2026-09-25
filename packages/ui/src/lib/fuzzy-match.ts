// Score bands match `crates/bronze-domain/src/fuzzy.rs`.

export const FUZZY_EXACT = 1_000_000;
export const FUZZY_PREFIX = 800_000;
export const FUZZY_SUBSTRING = 600_000;
export const FUZZY_SUBSEQUENCE = 400_000;
export const FUZZY_TYPO = 200_000;

const SUBSEQUENCE_MIN = 3;
const MAX_GAP = 1;
const TYPO_MIN = 4;

export function localeLower(text: string, locale = "en"): string {
  const value = String(text ?? "");
  const tag = String(locale ?? "").trim() || "en";
  try {
    return value.toLocaleLowerCase(tag);
  } catch {
    return value.toLocaleLowerCase("en");
  }
}

function codePoints(text: string): string[] {
  return Array.from(text);
}

function closeness(distance: number): number {
  return 1000 - Math.min(distance, 999);
}

function isAsciiLower(ch: string): boolean {
  return ch.length === 1 && ch >= "a" && ch <= "z";
}

function startsWith(
  hay: readonly string[],
  needle: readonly string[],
): boolean {
  if (needle.length > hay.length) {
    return false;
  }
  for (let index = 0; index < needle.length; index += 1) {
    if (hay[index] !== needle[index]) {
      return false;
    }
  }
  return true;
}

function indexOfSlice(
  hay: readonly string[],
  needle: readonly string[],
): number | null {
  if (needle.length === 0 || needle.length > hay.length) {
    return null;
  }
  const last = hay.length - needle.length;
  for (let start = 0; start <= last; start += 1) {
    let matches = true;
    for (let offset = 0; offset < needle.length; offset += 1) {
      if (hay[start + offset] !== needle[offset]) {
        matches = false;
        break;
      }
    }
    if (matches) {
      return start;
    }
  }
  return null;
}

function subsequenceBonus(
  hay: readonly string[],
  needle: readonly string[],
): number | null {
  if (needle.length < SUBSEQUENCE_MIN) {
    return null;
  }
  let from = 0;
  let prev = -1;
  let first = 0;
  let gaps = 0;
  for (const ch of needle) {
    let at = -1;
    for (let index = from; index < hay.length; index += 1) {
      if (hay[index] === ch) {
        at = index;
        break;
      }
    }
    if (at < 0) {
      return null;
    }
    if (prev >= 0) {
      const gap = at - prev - 1;
      if (gap > MAX_GAP) {
        return null;
      }
      gaps += gap;
    } else {
      first = at;
    }
    prev = at;
    from = at + 1;
  }
  return 999 - Math.min(first * 2 + gaps, 999);
}

function withinOneAsciiEdit(
  left: readonly string[],
  right: readonly string[],
): boolean {
  if (
    left.length === right.length &&
    left.every((ch, index) => ch === right[index])
  ) {
    return true;
  }
  if (Math.abs(left.length - right.length) > 1) {
    return false;
  }
  if (left.length === right.length) {
    let index = 0;
    let edits = 0;
    while (index < left.length) {
      if (left[index] === right[index]) {
        index += 1;
        continue;
      }
      edits += 1;
      if (edits > 1) {
        return false;
      }
      if (
        index + 1 < left.length &&
        left[index] === right[index + 1] &&
        left[index + 1] === right[index] &&
        isAsciiLower(left[index] ?? "") &&
        isAsciiLower(left[index + 1] ?? "") &&
        isAsciiLower(right[index] ?? "") &&
        isAsciiLower(right[index + 1] ?? "")
      ) {
        index += 2;
        continue;
      }
      if (
        !isAsciiLower(left[index] ?? "") ||
        !isAsciiLower(right[index] ?? "")
      ) {
        return false;
      }
      index += 1;
    }
    return edits === 1;
  }
  const shorter = left.length < right.length ? left : right;
  const longer = left.length < right.length ? right : left;
  let i = 0;
  let j = 0;
  let skips = 0;
  while (i < shorter.length && j < longer.length) {
    if (shorter[i] === longer[j]) {
      i += 1;
      j += 1;
      continue;
    }
    if (!isAsciiLower(longer[j] ?? "")) {
      return false;
    }
    skips += 1;
    if (skips > 1) {
      return false;
    }
    j += 1;
  }
  if (i !== shorter.length) {
    return false;
  }
  if (j < longer.length) {
    return (
      skips === 0 && longer.length - j === 1 && isAsciiLower(longer[j] ?? "")
    );
  }
  return skips <= 1;
}

function words(text: string): string[] {
  return text.match(/[\p{L}\p{N}]+/gu) ?? [];
}

function typoBonus(haystack: string, query: string): number | null {
  const queryPoints = codePoints(query);
  if (queryPoints.length < TYPO_MIN) {
    return null;
  }
  const hayPoints = codePoints(haystack);
  if (withinOneAsciiEdit(hayPoints, queryPoints)) {
    return closeness(0);
  }
  let best: number | null = null;
  let from = 0;
  for (const word of words(haystack)) {
    const at = haystack.indexOf(word, from);
    const start = at < 0 ? from : at;
    from = start + word.length;
    const wordPoints = codePoints(word);
    if (Math.abs(wordPoints.length - queryPoints.length) > 1) {
      continue;
    }
    if (!withinOneAsciiEdit(wordPoints, queryPoints)) {
      continue;
    }
    const charStart = codePoints(haystack.slice(0, start)).length;
    const bonus = closeness(charStart);
    if (best == null || bonus > best) {
      best = bonus;
    }
  }
  return best;
}

export function fuzzyMatchScore(
  haystack: string,
  query: string,
  locale = "en",
): number | null {
  const foldedQuery = localeLower(String(query ?? "").trim(), locale);
  if (foldedQuery.length === 0) {
    return 0;
  }
  const folded = localeLower(String(haystack ?? ""), locale);
  if (folded.length === 0) {
    return null;
  }
  if (folded === foldedQuery) {
    return FUZZY_EXACT;
  }
  const hay = codePoints(folded);
  const needle = codePoints(foldedQuery);
  if (startsWith(hay, needle)) {
    return FUZZY_PREFIX + closeness(hay.length - needle.length);
  }
  const at = indexOfSlice(hay, needle);
  if (at != null) {
    return FUZZY_SUBSTRING + closeness(at);
  }
  const sub = subsequenceBonus(hay, needle);
  if (sub != null) {
    return FUZZY_SUBSEQUENCE + sub;
  }
  const typo = typoBonus(folded, foldedQuery);
  if (typo == null) {
    return null;
  }
  return FUZZY_TYPO + typo;
}

export function fuzzyBestScore(
  parts: readonly string[],
  query: string,
  locale = "en",
): number | null {
  if (localeLower(String(query ?? "").trim(), locale).length === 0) {
    return 0;
  }
  let best: number | null = null;
  for (const part of parts) {
    const score = fuzzyMatchScore(part, query, locale);
    if (score != null && score > 0 && (best == null || score > best)) {
      best = score;
    }
  }
  return best;
}

export function fuzzyFilter<T>(
  items: readonly T[],
  query: string,
  locale: string | undefined,
  textOf: (item: T) => readonly string[] | string,
): T[] {
  const tag = locale?.trim() || "en";
  if (localeLower(String(query ?? "").trim(), tag).length === 0) {
    return items.slice();
  }
  const ranked: { item: T; score: number; index: number }[] = [];
  items.forEach((item, index) => {
    const raw = textOf(item);
    const parts = typeof raw === "string" ? [raw] : raw;
    const score = fuzzyBestScore(parts, query, tag);
    if (score != null && score > 0) {
      ranked.push({ item, score, index });
    }
  });
  ranked.sort(
    (left, right) => right.score - left.score || left.index - right.index,
  );
  return ranked.map((row) => row.item);
}
