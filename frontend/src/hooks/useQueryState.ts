import { useCallback, useEffect, useState } from "react";

/**
 * Mirrors a piece of state into the URL query string so filtered views are
 * linkable and survive back/forward. Uses replaceState so every keystroke
 * does not add a history entry.
 */
export function useQueryState<T>(
  parse: (params: URLSearchParams) => T,
  serialize: (value: T, params: URLSearchParams) => void,
) {
  const [value, setValue] = useState<T>(() =>
    parse(new URLSearchParams(window.location.search)),
  );

  useEffect(() => {
    const onNav = () =>
      setValue(parse(new URLSearchParams(window.location.search)));
    window.addEventListener("popstate", onNav);
    return () => window.removeEventListener("popstate", onNav);
  }, [parse]);

  const update = useCallback(
    (next: T | ((current: T) => T)) => {
      setValue((current) => {
        const resolved =
          typeof next === "function" ? (next as (c: T) => T)(current) : next;
        const params = new URLSearchParams();
        serialize(resolved, params);
        const query = params.toString();
        const suffix = query ? `?${query}` : "";
        window.history.replaceState(
          null,
          "",
          `${window.location.pathname}${suffix}`,
        );
        return resolved;
      });
    },
    [serialize],
  );

  return [value, update] as const;
}
