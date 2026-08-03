const TAG_COLOR_COUNT = 8;

export function bookTagColorClass(key: string): string {
  let hash = 0;
  for (const character of key) {
    hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
  }
  return `book-tag-color-${hash % TAG_COLOR_COUNT}`;
}

export function formatBookTagKey(key: string): string {
  const words = key.replaceAll(/[_-]+/g, " ").trim();
  if (!words) return key;
  return words[0].toLocaleUpperCase() + words.slice(1);
}
