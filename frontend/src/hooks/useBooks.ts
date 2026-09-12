import { useCallback, useEffect, useState } from "react";
import { fetchShelf, saveShelfReview, updateShelfStatus } from "../booksApi";
import type { AuthState } from "./useAuth";
import type { ShelfItem, ShelfStatus } from "../types";

export function useBooks(auth: AuthState) {
  const [books, setBooks] = useState<ShelfItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [savingId, setSavingId] = useState("");
  const [error, setError] = useState("");
  const isOwnerView = auth.status === "signedIn";

  const reload = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      setBooks(await fetchShelf());
    } catch (loadError) {
      setError((loadError as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const runUpdate = useCallback(
    async (item: ShelfItem, update: () => Promise<ShelfItem>) => {
      setSavingId(`${item.kind}:${item.id}`);
      setError("");
      try {
        const updated = await update();
        setBooks((current) =>
          current.map((entry) =>
            entry.id === updated.id && entry.kind === updated.kind
              ? updated
              : entry,
          ),
        );
      } catch (updateError) {
        setError((updateError as Error).message);
      } finally {
        setSavingId("");
      }
    },
    [],
  );

  const setStatus = useCallback(
    (item: ShelfItem, status: ShelfStatus) =>
      runUpdate(item, () => updateShelfStatus(item, status, auth.token)),
    [auth.token, runUpdate],
  );
  const saveReview = useCallback(
    (item: ShelfItem, rating: number, writeup: string) =>
      runUpdate(item, () => saveShelfReview(item, rating, writeup, auth.token)),
    [auth.token, runUpdate],
  );

  return {
    books,
    loading,
    savingId,
    error,
    isOwnerView,
    reload,
    setStatus,
    saveReview,
  };
}
