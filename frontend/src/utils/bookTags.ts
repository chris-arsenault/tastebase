const TAG_COLOR_COUNT = 8;

/**
 * Stable color slot per tag key. Known keys are pinned so "category" is the
 * same hue on every visit and across cards, chips, and filter rails; unknown
 * keys fall back to a hash so new keys still get a consistent color.
 */
const pinnedKeyColors: Record<string, number> = {
  category: 0,
  fit: 1,
  topic: 2,
  style: 3,
  outlook: 5,
};

function hashKey(key: string): number {
  let hash = 0;
  for (const character of key) {
    hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
  }
  return hash % TAG_COLOR_COUNT;
}

export function bookTagColorClass(key: string): string {
  const slot = pinnedKeyColors[key] ?? hashKey(key);
  return `book-tag-color-${slot}`;
}

export function formatBookTagKey(key: string): string {
  const words = key.replaceAll(/[_-]+/g, " ").trim();
  if (!words) return key;
  return words[0].toLocaleUpperCase() + words.slice(1);
}
