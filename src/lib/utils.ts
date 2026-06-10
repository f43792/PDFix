export function isPdf(path: string): boolean {
  return path.toLowerCase().endsWith('.pdf');
}

export function basename(path: string): string {
  const i = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return i >= 0 ? path.slice(i + 1) : path;
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let n = bytes / 1024;
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n < 10 ? n.toFixed(1) : n.toFixed(0)} ${units[i]}`;
}

export function formatDate(ms: number): string {
  if (!ms) return '';
  return new Date(ms).toLocaleString();
}

/**
 * Stateful matcher for an ordered, case-insensitive key sequence.
 * Returns a `feed(key)` function that yields true on a complete match.
 * Wrong key resets the match position, with a smart restart if the
 * wrong key happens to equal the first character of the sequence.
 */
export function makeSequenceMatcher(sequence: string[]) {
  const seq = sequence.map((c) => c.toLowerCase());
  let pos = 0;
  return {
    feed(key: string): boolean {
      const k = key.toLowerCase();
      if (k === seq[pos]) {
        pos++;
        if (pos === seq.length) {
          pos = 0;
          return true;
        }
      } else {
        pos = k === seq[0] ? 1 : 0;
      }
      return false;
    },
    reset() {
      pos = 0;
    },
  };
}
