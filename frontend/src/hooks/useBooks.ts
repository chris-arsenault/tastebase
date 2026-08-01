import { useCallback, useEffect, useState } from "react";
import {
  fetchBooks,
  fetchPublicBooks,
  saveBookReview,
  updateBookStatus,
  updateBookVisibility,
} from "../booksApi";
import type { AuthState } from "./useAuth";
import type { BookRecommendation, BookStatus } from "../types";

const errorMessage = (error: unknown) => (error as Error).message;

export function useBooks(auth: AuthState) {
  const [books, setBooks] = useState<BookRecommendation[]>([]);
  const [loading, setLoading] = useState(true);
  const [savingId, setSavingId] = useState("");
  const [error, setError] = useState("");
  const isOwnerView = auth.status === "signedIn";

  const reload = useCallback(async () => {
    if (auth.status === "loading") return;
    setLoading(true);
    setError("");
    try {
      const result = isOwnerView
        ? await fetchBooks(auth.token)
        : await fetchPublicBooks();
      setBooks(result);
    } catch (loadError) {
      setError(errorMessage(loadError));
    } finally {
      setLoading(false);
    }
  }, [auth.status, auth.token, isOwnerView]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const replaceBook = useCallback((book: BookRecommendation) => {
    setBooks((current) =>
      current.map((item) => (item.id === book.id ? book : item)),
    );
  }, []);

  const runUpdate = useCallback(
    async (id: string, update: () => Promise<BookRecommendation>) => {
      setSavingId(id);
      setError("");
      try {
        replaceBook(await update());
      } catch (updateError) {
        setError(errorMessage(updateError));
      } finally {
        setSavingId("");
      }
    },
    [replaceBook],
  );

  const setStatus = useCallback(
    (id: string, status: BookStatus) =>
      runUpdate(id, () => updateBookStatus(id, status, auth.token)),
    [auth.token, runUpdate],
  );
  const saveReview = useCallback(
    (id: string, rating: number, writeup: string) =>
      runUpdate(id, () => saveBookReview(id, rating, writeup, auth.token)),
    [auth.token, runUpdate],
  );
  const setVisibility = useCallback(
    (id: string, isPublic: boolean) =>
      runUpdate(id, () => updateBookVisibility(id, isPublic, auth.token)),
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
    setVisibility,
  };
}
