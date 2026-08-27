import { describe, it, expect } from 'vitest';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  collectCatalogErrors,
  extractPlaceholders,
  hasIcu,
  isValidBcp47Tag,
  looksConcatenated,
  runValidate,
} from '../scripts/validate';
import { pseudoArXB, pseudoEnXA } from '../scripts/pseudo';

const PKG = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function writeCatalogTree(tree: Record<string, Record<string, string>>): string {
  const dir = mkdtempSync(join(tmpdir(), 'bronze-i18n-'));
  for (const [tag, catalog] of Object.entries(tree)) {
    mkdirSync(join(dir, tag), { recursive: true });
    writeFileSync(join(dir, tag, 'app.json'), `${JSON.stringify(catalog, null, 2)}\n`);
  }
  return dir;
}

describe('extractPlaceholders', () => {
  it('takes only top-level ICU argument names', () => {
    expect(extractPlaceholders('{count, plural, =0 {0 items} one {1 item} other {# items}}')).toEqual([
      'count',
    ]);
    expect(extractPlaceholders('plain {foo} and {bar}')).toEqual(['bar', 'foo']);
  });
});

describe('hasIcu', () => {
  it('detects plural/select and not plain interpolation', () => {
    expect(hasIcu('{count, plural, one {1} other {#}}')).toBe(true);
    expect(hasIcu('From {appName}')).toBe(false);
  });
});

describe('collectCatalogErrors', () => {
  it('is empty when keys and ICU match', () => {
    const errors = collectCatalogErrors({
      en: {
        'queue.count': '{count, plural, =0 {0 items} one {1 item} other {# items}}',
        'app.name': 'Bronze',
      },
      'en-XA': {
        'queue.count': '{count, plural, =0 {0 items} one {1 item} other {# items}}',
        'app.name': '【BBrróoñnzzée】',
      },
    });
    expect(errors).toEqual([]);
  });

  it('reports missing keys', () => {
    const errors = collectCatalogErrors({
      en: { 'queue.count': '{count}', 'app.name': 'Bronze' },
      'en-XA': { 'app.name': '【B】' },
    });
    expect(errors.some((e) => e.includes('missing keys') && e.includes('queue.count'))).toBe(true);
  });

  it('reports ICU structure lost when plural collapses to a placeholder', () => {
    const errors = collectCatalogErrors({
      en: { 'queue.count': '{count, plural, =0 {0 items} one {1 item} other {# items}}' },
      'en-XA': { 'queue.count': '{count}' },
    });
    expect(errors.some((e) => e.includes('ICU structure lost'))).toBe(true);
  });

  it('reports placeholder name mismatch', () => {
    const errors = collectCatalogErrors({
      en: { 'capture.source': 'From {appName}' },
      'en-XA': { 'capture.source': 'From {name}' },
    });
    expect(errors.some((e) => e.includes('placeholder mismatch'))).toBe(true);
  });

  it('reports concatenated English values', () => {
    const errors = collectCatalogErrors({
      en: { 'help.body': 'First sentence. Second sentence' },
      'en-XA': { 'help.body': 'First sentence. Second sentence' },
    });
    expect(errors.some((e) => e.includes('looks concatenated'))).toBe(true);
  });

  it('still checks empty-string translations', () => {
    const errors = collectCatalogErrors({
      en: { 'capture.source': 'From {appName}' },
      'en-XA': { 'capture.source': '' },
    });
    expect(errors.some((e) => e.includes('placeholder mismatch'))).toBe(true);
  });
});

describe('looksConcatenated', () => {
  it('accepts seeded messages', () => {
    expect(looksConcatenated('Bronze')).toBe(false);
    expect(looksConcatenated('From {appName}')).toBe(false);
    expect(looksConcatenated('{count, plural, =0 {0 items} one {1 item} other {# items}}')).toBe(false);
  });
});

describe('isValidBcp47Tag', () => {
  it('accepts seeded tags and rejects junk', () => {
    expect(isValidBcp47Tag('en')).toBe(true);
    expect(isValidBcp47Tag('en-XA')).toBe(true);
    expect(isValidBcp47Tag('ar-XB')).toBe(true);
    expect(isValidBcp47Tag('zh-Hant-HK')).toBe(true);
    expect(isValidBcp47Tag('!!!')).toBe(false);
    expect(isValidBcp47Tag('not a tag')).toBe(false);
  });
});

describe('runValidate', () => {
  it('accepts the seeded locale tree', () => {
    const { ok, errors } = runValidate();
    expect(errors).toEqual([]);
    expect(ok).toBe(true);
  });

  it('fails when a locale omits a base key', () => {
    const dir = writeCatalogTree({
      en: { 'queue.count': '{count}', 'app.name': 'Bronze' },
      'en-XA': { 'app.name': '【B】' },
    });
    try {
      const { ok, errors } = runValidate(dir);
      expect(ok).toBe(false);
      expect(errors.some((e) => e.includes('missing keys') && e.includes('queue.count'))).toBe(true);
    } finally {
      rmSync(dir, { recursive: true });
    }
  });

  it('fails when ICU plural collapses to a placeholder', () => {
    const dir = writeCatalogTree({
      en: { 'queue.count': '{count, plural, =0 {0 items} one {1 item} other {# items}}' },
      'en-XA': { 'queue.count': '{count}' },
    });
    try {
      const { ok, errors } = runValidate(dir);
      expect(ok).toBe(false);
      expect(errors.some((e) => e.includes('ICU structure lost') || e.includes('placeholder mismatch'))).toBe(true);
    } finally {
      rmSync(dir, { recursive: true });
    }
  });

  it('fails on an invalid BCP 47 directory name', () => {
    const dir = writeCatalogTree({
      en: { 'app.name': 'Bronze' },
      x: { 'app.name': 'Bronze' },
    });
    try {
      const { ok, errors } = runValidate(dir);
      expect(ok).toBe(false);
      expect(errors.some((e) => e.includes('invalid tag'))).toBe(true);
    } finally {
      rmSync(dir, { recursive: true });
    }
  });
});

describe('pseudoEnXA', () => {
  it('expands length, marks bounds, accents letters, and keeps placeholders verbatim', () => {
    const src = 'From {appName}';
    const out = pseudoEnXA(src);
    expect(out.startsWith('【')).toBe(true);
    expect(out.endsWith('】')).toBe(true);
    expect(out).toContain('{appName}');
    expect(out).toMatch(/[áéíóúñçÁÉÍÓÚ]/);
    expect(out.length).toBeGreaterThanOrEqual(Math.ceil(src.length * 1.4));
  });
});

describe('pseudoArXB', () => {
  it('preserves simple placeholders inside bidi isolates', () => {
    const out = pseudoArXB('From {appName}');
    expect(out).toContain('\u2067{appName}\u2069');
    expect(out).not.toContain('{emanppa}');
  });

  it('wraps each repeated simple placeholder once', () => {
    const out = pseudoArXB('{appName} {appName}');
    const wrapped = out.match(/\u2067\{appName\}\u2069/g) ?? [];
    expect(wrapped).toHaveLength(2);
    expect(out).not.toContain('\u2067\u2067');
  });
});

describe('committed pseudo catalogs', () => {
  it('match the generators', () => {
    const en = JSON.parse(readFileSync(resolve(PKG, 'locales/en/app.json'), 'utf8')) as Record<string, string>;
    const xa = JSON.parse(readFileSync(resolve(PKG, 'locales/en-XA/app.json'), 'utf8')) as Record<string, string>;
    const xb = JSON.parse(readFileSync(resolve(PKG, 'locales/ar-XB/app.json'), 'utf8')) as Record<string, string>;
    for (const [k, v] of Object.entries(en)) {
      expect(xa[k]).toBe(pseudoEnXA(v));
      expect(xb[k]).toBe(pseudoArXB(v));
    }
  });
});
