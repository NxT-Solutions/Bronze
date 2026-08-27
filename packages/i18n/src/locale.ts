export type SupportedLocale = 'en' | 'en-XA' | 'ar-XB';
export type FallbackChain = readonly string[];

const SUPPORTED: SupportedLocale[] = ['en', 'en-XA', 'ar-XB'];

export function canonicalizeBcp47(tag: string): string {
  try {
    const normalized = tag.replace(/_/g, '-');
    const canon = Intl.getCanonicalLocales(normalized);
    return canon[0] ?? 'und';
  } catch {
    return 'und';
  }
}

export function isSupported(tag: string): tag is SupportedLocale {
  const canon = canonicalizeBcp47(tag);
  return (SUPPORTED as string[]).includes(canon);
}

function splitSubtags(tag: string): string[] {
  return tag.split(/[-_]/).filter(Boolean);
}

export function computeFallbackChain(requested: string): FallbackChain {
  const canon = canonicalizeBcp47(requested);
  if (!canon || canon === 'und') {
    return ['en'];
  }
  if (canon === 'en') {
    return ['en'];
  }

  const chain: string[] = [];
  let current = canon;

  while (current && current !== 'en') {
    if (chain.includes(current)) break;
    chain.push(current);
    const parts = splitSubtags(current);
    if (parts.length <= 1) break;

    const next = parts.slice(0, -1).join('-');
    if (!next || next === current) break;
    current = next;
  }

  if (!chain.includes('en')) {
    chain.push('en');
  }

  // dedup preserve order
  const seen = new Set<string>();
  const final: string[] = [];
  for (const c of chain) {
    const cc = canonicalizeBcp47(c);
    if (!seen.has(cc)) {
      seen.add(cc);
      final.push(cc);
    }
  }
  return final as unknown as FallbackChain;
}
