import { describe, it, expect } from 'vitest';
import {
  isPdf,
  basename,
  formatSize,
  formatDate,
  makeSequenceMatcher,
} from './utils';

describe('isPdf', () => {
  it('accepts .pdf regardless of case', () => {
    expect(isPdf('foo.pdf')).toBe(true);
    expect(isPdf('FOO.PDF')).toBe(true);
    expect(isPdf('mix.PdF')).toBe(true);
  });

  it('rejects non-pdf and pdf-in-middle', () => {
    expect(isPdf('foo.txt')).toBe(false);
    expect(isPdf('foo.pdf.txt')).toBe(false);
    expect(isPdf('justpdf')).toBe(false);
    expect(isPdf('')).toBe(false);
  });
});

describe('basename', () => {
  it('strips unix-style paths', () => {
    expect(basename('/a/b/c.pdf')).toBe('c.pdf');
  });

  it('strips windows-style paths', () => {
    expect(basename('C:\\a\\b\\c.pdf')).toBe('c.pdf');
  });

  it('returns input when no separator present', () => {
    expect(basename('c.pdf')).toBe('c.pdf');
  });

  it('handles mixed separators (windows-mixed)', () => {
    expect(basename('C:\\a/b\\c.pdf')).toBe('c.pdf');
  });
});

describe('formatSize', () => {
  it('returns bytes below 1KB', () => {
    expect(formatSize(0)).toBe('0 B');
    expect(formatSize(512)).toBe('512 B');
  });

  it('uses one decimal under 10 of a unit', () => {
    expect(formatSize(2 * 1024)).toBe('2.0 KB');
    expect(formatSize(9.5 * 1024)).toBe('9.5 KB');
  });

  it('drops decimals at or above 10', () => {
    expect(formatSize(50 * 1024)).toBe('50 KB');
  });

  it('promotes through KB, MB, GB, TB', () => {
    expect(formatSize(1024 * 1024)).toMatch(/MB$/);
    expect(formatSize(1024 ** 3)).toMatch(/GB$/);
    expect(formatSize(1024 ** 4)).toMatch(/TB$/);
  });
});

describe('formatDate', () => {
  it('returns empty string for falsy ms', () => {
    expect(formatDate(0)).toBe('');
  });

  it('produces a non-empty string for valid ms', () => {
    expect(formatDate(Date.parse('2024-01-15T12:00:00Z'))).not.toBe('');
  });
});

describe('makeSequenceMatcher', () => {
  it('returns true only when full sequence is fed', () => {
    const m = makeSequenceMatcher(['f', 'c', 'n']);
    expect(m.feed('f')).toBe(false);
    expect(m.feed('c')).toBe(false);
    expect(m.feed('n')).toBe(true);
  });

  it('is case insensitive', () => {
    const m = makeSequenceMatcher(['f', 'c', 'n']);
    expect(m.feed('F')).toBe(false);
    expect(m.feed('C')).toBe(false);
    expect(m.feed('N')).toBe(true);
  });

  it('resets on wrong key, then needs full sequence again', () => {
    const m = makeSequenceMatcher(['f', 'c', 'n']);
    m.feed('f');
    m.feed('c');
    m.feed('x'); // breaks
    expect(m.feed('n')).toBe(false); // would have completed but we reset
  });

  it('smart-restarts when wrong key happens to match the first character', () => {
    const m = makeSequenceMatcher(['f', 'c', 'n']);
    m.feed('f'); // pos=1
    m.feed('f'); // wrong (expected 'c'), but matches first → pos=1 again
    expect(m.feed('c')).toBe(false);
    expect(m.feed('n')).toBe(true);
  });

  it('reset() clears progress', () => {
    const m = makeSequenceMatcher(['f', 'c', 'n']);
    m.feed('f');
    m.feed('c');
    m.reset();
    expect(m.feed('n')).toBe(false);
  });

  it('cycles after a successful match', () => {
    const m = makeSequenceMatcher(['a', 'b']);
    expect(m.feed('a')).toBe(false);
    expect(m.feed('b')).toBe(true);
    // pos resets after match — we should be able to match again
    expect(m.feed('a')).toBe(false);
    expect(m.feed('b')).toBe(true);
  });
});
