import { useCallback, useEffect, useState } from "react";
import { fetchRecipe } from "../api";
import type { RecipeFull, RecipeIngredient } from "../types";

export function slugify(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}

export function navigateToRecipe(slug: string) {
  window.history.pushState(null, "", `/recipes/${slug}`);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

export const sourceLabels: Record<string, string> = {
  claude: "Recipe by Claude",
  manual: "Manual",
  import: "Imported",
};

export const formatTimer = (s: number) => {
  const m = Math.floor(s / 60),
    r = s % 60;
  if (s < 60) return `${s}s`;
  return r === 0 ? `${m}m` : `${m}m ${r}s`;
};

export function renderMarkdown(text: string): React.ReactNode[] {
  const unescaped = text.replace(/\\n/g, "\n");
  const paragraphs = unescaped.split(/\n\n/);
  return paragraphs.map((para, pi) => {
    const lines = para.split(/\n/);
    const children: React.ReactNode[] = [];
    lines.forEach((line, li) => {
      if (li > 0) children.push(<br key={`br-${pi}-${li}`} />);
      const parts = line.split(/(\*\*[^*]+\*\*)/g);
      parts.forEach((part, partIdx) => {
        const boldMatch = /^\*\*(.+)\*\*$/.exec(part);
        if (boldMatch) {
          children.push(
            <strong key={`b-${pi}-${li}-${partIdx}`}>{boldMatch[1]}</strong>,
          );
        } else {
          children.push(part);
        }
      });
    });
    return (
      <p key={`p-${pi}`} className="recipe-notes-paragraph">
        {children}
      </p>
    );
  });
}

export const formatAmount = (n: number): string => {
  if (Number.isInteger(n)) return n.toString();
  const frac = n % 1;
  const whole = Math.floor(n);
  const fractions: [number, string][] = [
    [0.25, "\u00BC"],
    [0.333, "\u2153"],
    [0.5, "\u00BD"],
    [0.666, "\u2154"],
    [0.75, "\u00BE"],
  ];
  for (const [val, sym] of fractions) {
    if (Math.abs(frac - val) < 0.02) return whole > 0 ? `${whole}${sym}` : sym;
  }
  return n.toFixed(1).replace(/\.0$/, "");
};

export function useRecipeFetch(recipeId: string) {
  const [recipe, setRecipe] = useState<RecipeFull | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [fetchKey, setFetchKey] = useState(0);

  useEffect(() => {
    let stale = false;
    fetchRecipe(recipeId)
      .then((data) => {
        if (!stale) {
          setRecipe(data);
          setLoading(false);
        }
      })
      .catch((e: unknown) => {
        if (!stale) {
          setError((e as Error).message);
          setLoading(false);
        }
      });
    return () => {
      stale = true;
    };
  }, [recipeId, fetchKey]);

  const reload = useCallback(() => setFetchKey((k) => k + 1), []);
  return { recipe, loading, error, reload };
}

type IngredientInfo = { name: string; linkedRecipeId?: string | null };

export function buildIngredientMap(
  ingredients: RecipeIngredient[],
): Map<string, IngredientInfo> {
  const map = new Map<string, IngredientInfo>();
  for (const ing of ingredients)
    map.set(ing.widgetId, {
      name: ing.shortName || ing.name,
      linkedRecipeId: ing.linkedRecipeId,
    });
  return map;
}

export function navigateToLinkedRecipe(
  linkedId: string,
  cache: React.MutableRefObject<Map<string, RecipeFull>>,
) {
  const cached = cache.current.get(linkedId);
  if (cached) {
    navigateToRecipe(slugify(cached.title));
    return;
  }
  fetchRecipe(linkedId)
    .then((data) => {
      cache.current.set(linkedId, data);
      navigateToRecipe(slugify(data.title));
    })
    .catch(() => {});
}

export function resolveTokens(
  content: string,
  ingredientMap: Map<string, IngredientInfo>,
  linkedRecipeCache: React.MutableRefObject<Map<string, RecipeFull>>,
): React.ReactNode[] {
  const result: React.ReactNode[] = [];
  let lastIndex = 0;
  const re = /\{(\w+)\}/g;
  let match = re.exec(content);
  while (match !== null) {
    if (match.index > lastIndex)
      result.push(content.slice(lastIndex, match.index));
    const info = ingredientMap.get(match[1]);
    if (info) {
      if (info.linkedRecipeId) {
        const linkedId = info.linkedRecipeId;
        const cached = linkedRecipeCache.current.get(linkedId);
        result.push(
          <a
            key={match.index}
            href={cached ? `/recipes/${slugify(cached.title)}` : "#"}
            className="recipe-ingredient-token recipe-ingredient-token-linked"
            onClick={(e) => {
              e.preventDefault();
              navigateToLinkedRecipe(linkedId, linkedRecipeCache);
            }}
          >
            {info.name}
          </a>,
        );
      } else {
        result.push(
          <strong key={match.index} className="recipe-ingredient-token">
            {info.name}
          </strong>,
        );
      }
    } else {
      result.push(match[0]);
    }
    lastIndex = re.lastIndex;
    match = re.exec(content);
  }
  if (lastIndex < content.length) result.push(content.slice(lastIndex));
  return result;
}

function LinkedTooltip({ preview }: Readonly<{ preview: RecipeFull | null }>) {
  const thumbUrl = preview?.images?.[0]?.imageUrl;
  if (!preview) {
    return <span className="recipe-linked-tooltip-loading">{"\u2026"}</span>;
  }
  return (
    <>
      {thumbUrl && (
        <img className="recipe-linked-tooltip-thumb" src={thumbUrl} alt="" />
      )}
      <div className="recipe-linked-tooltip-info">
        <span className="recipe-linked-tooltip-title">{preview.title}</span>
        {preview.description && (
          <span className="recipe-linked-tooltip-desc">
            {preview.description}
          </span>
        )}
        <span className="recipe-linked-tooltip-meta">
          {preview.ingredients.length} ingredient
          {preview.ingredients.length !== 1 ? "s" : ""} {"\u00B7"}{" "}
          {preview.baseServings} serving{preview.baseServings !== 1 ? "s" : ""}
        </span>
      </div>
    </>
  );
}

export function LinkedIngredientRow({
  ing,
  scale,
  cache,
}: Readonly<{
  ing: RecipeIngredient;
  scale: number;
  cache: React.MutableRefObject<Map<string, RecipeFull>>;
}>) {
  const linkedId = ing.linkedRecipeId!;
  const [preview, setPreview] = useState<RecipeFull | null>(
    () => cache.current.get(linkedId) ?? null,
  );

  useEffect(() => {
    if (preview || cache.current.has(linkedId)) return;
    let stale = false;
    fetchRecipe(linkedId)
      .then((data) => {
        cache.current.set(linkedId, data);
        if (!stale) setPreview(data);
      })
      .catch(() => {});
    return () => {
      stale = true;
    };
  }, [linkedId, preview, cache]);

  return (
    <li className="recipe-ing-linked-row">
      <span className="recipe-ing-amount">
        {formatAmount(ing.amount * scale)} {ing.unit}
      </span>
      <a
        href={preview ? `/recipes/${slugify(preview.title)}` : "#"}
        className="recipe-ing-linked"
        onClick={(e) => {
          e.preventDefault();
          navigateToLinkedRecipe(linkedId, cache);
        }}
      >
        {ing.name}
        <span className="recipe-ing-linked-icon" aria-hidden="true">
          {"\u2197"}
        </span>
      </a>
      <div className="recipe-linked-tooltip">
        <LinkedTooltip preview={preview} />
      </div>
    </li>
  );
}
